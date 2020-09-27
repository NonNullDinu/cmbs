use std::{
    collections::HashMap,
    error::Error,
    ops::Deref,
    path::{Path, PathBuf},
};

use lalrpop_util::ParseError;

use libutils::toolchain::c::{get_c_toolchain, CTc};
use libutils::toolchain::cpp::{get_cpp_toolchain, CPPTc};
use libutils::toolchain::{
    CCompiler, CPPCompiler, CPPToolchain, CPPToolchainLinker, CToolchain, CToolchainLinker,
};

use crate::{
    grammar::{self, ast::*, TokLoc},
    handle::Handle,
    interpreter::{
        diagnostics::{errors::*, push_diagnostic_ctx, warnings::*, DiagnosticsCtx, Location},
        types::*,
    },
};

#[path = "diagnostics/diagnostics.rs"]
pub(crate) mod diagnostics;
pub(crate) mod gen;
pub(crate) mod ops;
pub(crate) mod prelude_values;
pub(crate) mod types;

pub(crate) const DOCS_ROOT: &str = "https://leafbuild.gitlab.io/docs/";

pub struct EnvConfig {
    angry_errors_enabled: bool,

    error_cascade_enabled: bool,
    signal_build_failure: bool,

    output_directory: PathBuf,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self {
            angry_errors_enabled: false,
            error_cascade_enabled: true,
            signal_build_failure: false,
            output_directory: PathBuf::from("."),
        }
    }

    pub fn set_angry_errors(&mut self, enabled: bool) -> &mut EnvConfig {
        self.angry_errors_enabled = enabled;
        self
    }

    pub fn set_error_cascade(&mut self, enabled: bool) -> &mut EnvConfig {
        self.error_cascade_enabled = enabled;
        self
    }

    pub fn set_output_directory(&mut self, output_directory: PathBuf) -> &mut EnvConfig {
        self.output_directory = output_directory;
        self
    }

    pub fn set_signal_build_failure(&mut self, signal_build_failure: bool) -> &mut EnvConfig {
        self.signal_build_failure = signal_build_failure;
        self
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct EnvImut {
    call_pools: CallPoolsWrapper,
    config: EnvConfig,

    prelude_values: HashMap<String, Value<Box<dyn ValueTypeMarker>>>,
}

pub(crate) struct EnvModData {
    mod_id: usize,
    path: PathBuf,
}

impl EnvModData {
    pub(crate) fn new(mod_id: usize, path: PathBuf) -> Self {
        Self { mod_id, path }
    }
}

pub(crate) struct EnvMut {
    /// the current task id we are at, universally unique
    task_id: usize,

    /// the current module id we are at, universally unique
    mod_id: usize,

    modules: Vec<EnvModData>,

    /// the C toolchain
    cc: Option<CTc>,
    /// the C++ toolchain
    cxx: Option<CPPTc>,

    tasks: Vec<Box<dyn LeafTask>>,

    diagnostics_ctx: DiagnosticsCtx,
}

impl EnvMut {
    pub(crate) fn get_and_cache_cc(&mut self) -> &CTc {
        if self.cc.is_none() {
            let cc = get_c_toolchain().expect("Cannot find CC");
            self.cc = Some(cc);
        }
        self.cc.as_ref().unwrap()
    }

    pub(crate) fn get_and_cache_cxx(&mut self) -> &CPPTc {
        if self.cxx.is_none() {
            let cxx = get_cpp_toolchain().expect("Cannot find CXX");
            self.cxx = Some(cxx);
        }
        self.cxx.as_ref().unwrap()
    }

    pub(crate) fn get_cached_cc(&self) -> &CTc {
        self.cc.as_ref().unwrap()
    }

    pub(crate) fn get_cached_cxx(&self) -> &CPPTc {
        self.cxx.as_ref().unwrap()
    }
}

pub(crate) struct Env {
    imut: EnvImut,
    mut_: EnvMut,
}

impl Env {
    pub(crate) fn new(cfg: EnvConfig) -> Self {
        Self {
            mut_: EnvMut {
                diagnostics_ctx: DiagnosticsCtx::new(
                    cfg.angry_errors_enabled,
                    cfg.error_cascade_enabled,
                    cfg.signal_build_failure,
                ),
                task_id: 0,
                mod_id: 1,
                cc: None,
                cxx: None,
                tasks: vec![],
                modules: vec![],
            },
            imut: EnvImut {
                call_pools: CallPoolsWrapper::new(),
                config: cfg,
                prelude_values: prelude_values::get_prelude_values(),
            },
        }
    }

    pub(crate) fn write_results(&mut self) -> Result<(), Box<dyn Error>> {
        let buf = PathBuf::from(self.imut.config.output_directory.clone());
        if !buf.exists() {
            std::fs::create_dir(buf.as_path())?;
        }

        gen::ninja::write_to(self, buf)
    }

    pub(crate) fn get_root_path_for_module(&self, mod_id: usize) -> Option<&PathBuf> {
        let path = &self
            .mut_
            .modules
            .iter()
            .find(|module| module.mod_id == mod_id)?
            .path;
        Some(path)
    }
}

pub(crate) struct ProjectData {
    name: String,
    mod_id: usize,
}

pub(crate) struct ModuleData {
    name: String,
    mod_id: usize,
}

pub(crate) enum EnvFrameType {
    Workspace,
    Project(ProjectData),
    Module(ModuleData),
    Unknown, // default value, uninitialized
}

pub(crate) struct EnvFrame<'env> {
    env_ref: &'env EnvImut,
    env_mut_ref: &'env mut EnvMut,
    variables: HashMap<String, Variable<Box<dyn ValueTypeMarker>>>,
    env_frame_data: EnvFrameData,
    file_id: usize,
    fr_type: EnvFrameType,
    root_path: PathBuf,
}

impl<'env> EnvFrame<'env> {
    pub(crate) fn get_value_for_variable(
        &self,
        id: &str,
    ) -> Option<&Value<Box<dyn ValueTypeMarker>>> {
        if let Some(v) = self.env_ref.prelude_values.get(id) {
            return Some(v);
        }
        self.variables
            .iter()
            .find(|&(var_name, _)| var_name == id)
            .map(|var| var.1.get_value())
    }

    pub(crate) fn get_diagnostics_ctx(&'env self) -> &'env DiagnosticsCtx {
        &self.env_mut_ref.diagnostics_ctx
    }

    pub(crate) fn get_pools_wrapper(&self) -> &'env CallPoolsWrapper {
        &self.env_ref.call_pools
    }

    pub(crate) fn get_variables_mut(
        &mut self,
    ) -> &mut HashMap<String, Variable<Box<dyn ValueTypeMarker>>> {
        &mut self.variables
    }

    pub(crate) fn get_file_id(&self) -> usize {
        self.file_id
    }

    pub(crate) fn get_mod_id(&self) -> usize {
        match self.fr_type {
            EnvFrameType::Workspace => 0,
            EnvFrameType::Project(ProjectData { mod_id, .. })
            | EnvFrameType::Module(ModuleData { mod_id, .. }) => mod_id,
            EnvFrameType::Unknown => 0,
        }
    }

    pub(crate) fn allocate_new_task(&mut self) -> usize {
        let new_task_id = self.env_mut_ref.task_id + 1;
        std::mem::replace(&mut self.env_mut_ref.task_id, new_task_id)
    }

    pub(crate) fn next_mod_id(&mut self) -> usize {
        let new_mod_id = self.env_mut_ref.mod_id + 1;
        std::mem::replace(&mut self.env_mut_ref.mod_id, new_mod_id)
    }
}

pub struct EnvFrameData {
    mod_id: usize,
    root_path: PathBuf,

    tasks: Vec<Box<dyn LeafTask>>,
}

pub(crate) struct EnvFrameReturns {
    mod_id: usize,
    root_path: PathBuf,

    tasks: Vec<Box<dyn LeafTask>>,
}

impl EnvFrameData {
    pub(crate) fn empty(root_path: PathBuf) -> Self {
        Self {
            tasks: vec![],
            mod_id: 0,
            root_path,
        }
    }
}

impl From<EnvFrameData> for EnvFrameReturns {
    fn from(r: EnvFrameData) -> Self {
        Self {
            mod_id: r.mod_id,
            root_path: r.root_path,
            tasks: r.tasks,
        }
    }
}

impl EnvFrameReturns {
    fn apply_changes_to_env_struct(self, env: &mut Env) {
        self.apply_changes_to_env((&env.imut, &mut env.mut_))
    }
    fn apply_changes_to_env(self, env: (&EnvImut, &mut EnvMut)) {
        self.tasks
            .into_iter()
            .for_each(|task| env.1.tasks.push(task));

        env.1
            .modules
            .push(EnvModData::new(self.mod_id, self.root_path))
    }
}

pub(crate) struct Variable<T>
where
    T: ValueTypeMarker,
{
    name: String,
    value: Value<T>,
}

impl<T> Variable<T>
where
    T: ValueTypeMarker + Sized,
{
    pub(crate) fn new(name: String, value: Value<T>) -> Self {
        Self { name, value }
    }

    pub(crate) fn get_value(&self) -> &Value<T> {
        &self.value
    }

    pub(crate) fn get_value_mut(&mut self) -> &mut Value<T> {
        &mut self.value
    }
}

pub(crate) trait ValueTypeMarker {
    fn stringify(&self) -> String;
    fn clone_to_value(&self) -> Value<Box<dyn ValueTypeMarker>>;
    fn get_type_id(&self) -> types::TypeId;
    fn get_type_id_and_value(&self) -> types::TypeIdAndValue;

    fn get_type_id_and_value_required(
        &self,
        required_type: TypeId,
    ) -> Result<TypeIdAndValue, TypeId> {
        let r = self.get_type_id_and_value();
        let tp = r.degrade();
        if tp == required_type {
            Ok(r)
        } else {
            Err(tp)
        }
    }
}

impl<T> ValueTypeMarker for Box<T>
where
    T: ValueTypeMarker + ?Sized,
{
    fn stringify(&self) -> String {
        self.deref().stringify()
    }

    fn clone_to_value(&self) -> Value<Box<dyn ValueTypeMarker>> {
        self.deref().clone_to_value()
    }

    fn get_type_id(&self) -> TypeId {
        self.deref().get_type_id()
    }

    fn get_type_id_and_value(&self) -> types::TypeIdAndValue {
        self.deref().get_type_id_and_value()
    }
}

impl<T> ValueTypeMarker for &mut Box<T>
where
    T: ValueTypeMarker + ?Sized,
{
    fn stringify(&self) -> String {
        self.deref().stringify()
    }

    fn clone_to_value(&self) -> Value<Box<dyn ValueTypeMarker>> {
        self.deref().clone_to_value()
    }

    fn get_type_id(&self) -> TypeId {
        self.deref().get_type_id()
    }

    fn get_type_id_and_value(&self) -> types::TypeIdAndValue {
        self.deref().get_type_id_and_value()
    }
}

pub(crate) struct Value<T>
where
    T: ValueTypeMarker,
{
    base_type_id: TypeId,
    value: T,
    creation_location: Location,
}

impl<T> Value<T>
where
    T: ValueTypeMarker,
{
    pub fn new(value: T) -> Self {
        let base_type_id = value.get_type_id();
        Self {
            value,
            base_type_id,
            creation_location: 0..1,
        }
    }

    pub(crate) fn with_location(mut self, location: Location) -> Self {
        self.creation_location = location;
        self
    }

    pub(crate) fn get_base_type(&self) -> &TypeId {
        &self.base_type_id
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }
}

impl<T> ValueTypeMarker for Value<T>
where
    T: ValueTypeMarker,
{
    fn stringify(&self) -> String {
        self.value.stringify()
    }

    fn clone_to_value(&self) -> Value<Box<dyn ValueTypeMarker>> {
        self.value.clone_to_value()
    }

    fn get_type_id(&self) -> TypeId {
        self.value.get_type_id()
    }

    fn get_type_id_and_value(&self) -> TypeIdAndValue {
        self.value.get_type_id_and_value()
    }
}

pub(crate) struct LaterValue<'a> {
    val_expr: &'a Expr,
}

impl<'a> LaterValue<'a> {
    pub(crate) fn new(val_expr: &'a Expr) -> Self {
        Self { val_expr }
    }
    pub(crate) fn compute(&self, frame: &mut EnvFrame) -> Value<Box<dyn ValueTypeMarker>> {
        self.val_expr.eval_in_env(frame)
    }
}

/// A mutable value reference
pub(crate) struct ValRefMut<'a, T>
where
    T: ValueTypeMarker,
{
    reference: &'a mut Value<T>,
}

impl<'a, T> ValRefMut<'a, T>
where
    T: ValueTypeMarker,
{
    pub(crate) fn new(reference: &'a mut Value<T>) -> Self {
        Self { reference }
    }
}

impl<'a, T> ValueTypeMarker for ValRefMut<'a, T>
where
    T: ValueTypeMarker,
{
    fn stringify(&self) -> String {
        self.reference.stringify()
    }

    fn clone_to_value(&self) -> Value<Box<dyn ValueTypeMarker>> {
        // when you clone a reference, it should return a brand new object with the same value
        self.reference.get_value().clone_to_value()
    }

    fn get_type_id(&self) -> TypeId {
        self.reference.get_value().get_type_id()
    }

    fn get_type_id_and_value(&self) -> TypeIdAndValue<'_> {
        self.reference.get_value().get_type_id_and_value()
    }
}

pub(crate) fn add_file(file: String, src: String, env: &mut Env) -> usize {
    add_file_ctx(file, src, &mut env.mut_.diagnostics_ctx)
}

pub(crate) fn add_file_ctx(file: String, src: String, ctx: &mut DiagnosticsCtx) -> usize {
    ctx.new_file(file, src)
}

pub(crate) fn interpret<'env>(
    env: &'env mut Env,
    program: &'_ AstProgram,
    file_id: usize,
    root_path: PathBuf,
) {
    let statements = program.get_statements();
    let mut frame = EnvFrame {
        variables: HashMap::new(),
        env_frame_data: EnvFrameData::empty(root_path.clone()),
        env_ref: &env.imut,
        env_mut_ref: &mut env.mut_,
        file_id,
        fr_type: EnvFrameType::Unknown,
        root_path,
    };

    statements.iter().for_each(|statement| {
        run_in_env_frame(statement, &mut frame);
    });

    let efr = EnvFrameReturns::from(frame.env_frame_data);
    efr.apply_changes_to_env_struct(env);
}

pub fn start_on(proj_path: &Path, handle: &mut Handle) {
    let path = proj_path.join("build.leaf");
    let path_clone = path.clone();
    let src = String::from_utf8(std::fs::read(path).unwrap()).unwrap();
    let src_len = src.len();
    let result = grammar::parse(&src);
    let file_id = add_file(
        path_clone.to_str().unwrap().to_string(),
        src.clone(),
        &mut handle.env,
    );
    match result {
        Ok(program) => {
            interpret(&mut handle.env, &program, file_id, PathBuf::from(proj_path));
            handle.write_results();
        }
        Err(e) => {
            let syntax_error = match e {
                ParseError::InvalidToken { location } => {
                    SyntaxError::new(location..location + 1, "invalid token")
                }
                ParseError::UnrecognizedEOF { location, expected } => SyntaxError::new(
                    location..location + 1,
                    format!("unrecognized EOF, expected {:?}", expected),
                ),
                ParseError::UnrecognizedToken { token, expected } => SyntaxError::new(
                    token.0..token.2,
                    format!("Unexpected token {}, expected {:?}", token.1, expected),
                ),
                ParseError::ExtraToken { token } => {
                    SyntaxError::new(token.0..token.2, format!("extra token: {}", token.1))
                }
                ParseError::User { error } => SyntaxError::new(0..1, error),
            };
            push_diagnostic_ctx(syntax_error, &handle.env.mut_.diagnostics_ctx)
        }
    }
}

// code to load and work with subdirectories

pub(crate) fn interpret_subdir<'env>(
    env: (&'env EnvImut, &'env mut EnvMut),
    program: &'_ AstProgram,
    file_id: usize,
    root_path: PathBuf,
) {
    let statements = program.get_statements();
    let mut frame = EnvFrame {
        variables: HashMap::new(),
        env_frame_data: EnvFrameData::empty(root_path.clone()),
        env_ref: env.0,
        env_mut_ref: env.1,
        file_id,
        fr_type: EnvFrameType::Unknown,
        root_path,
    };

    statements.iter().for_each(|statement| {
        run_in_env_frame(statement, &mut frame);
    });

    let efr = EnvFrameReturns::from(frame.env_frame_data);
    efr.apply_changes_to_env(env);
}

pub(crate) fn start_on_subdir(root_path: &Path, env: (&EnvImut, &mut EnvMut)) {
    let path = root_path.join("build.leaf");
    let path_clone = path.clone();
    let src = String::from_utf8(std::fs::read(path).unwrap()).unwrap();
    let src_len = src.len();
    let result = grammar::parse(&src);
    let file_id = add_file_ctx(
        path_clone.to_str().unwrap().to_string(),
        src.clone(),
        &mut env.1.diagnostics_ctx,
    );
    match result {
        Ok(program) => {
            interpret_subdir(env, &program, file_id, PathBuf::from(root_path));
        }
        Err(e) => {
            let syntax_error = match e {
                ParseError::InvalidToken { location } => {
                    SyntaxError::new(location..location + 1, "invalid token")
                }
                ParseError::UnrecognizedEOF { location, expected } => SyntaxError::new(
                    location..location + 1,
                    format!("unrecognized EOF, expected {:?}", expected),
                ),
                ParseError::UnrecognizedToken { token, expected } => SyntaxError::new(
                    token.0..token.2,
                    format!("Unexpected token {}, expected {:?}", token.1, expected),
                ),
                ParseError::ExtraToken { token } => {
                    SyntaxError::new(token.0..token.2, format!("extra token: {}", token.1))
                }
                ParseError::User { error } => SyntaxError::new(0..1, error),
            };
            push_diagnostic_ctx(syntax_error, &env.1.diagnostics_ctx)
        }
    }
}

pub(crate) struct CallPoolsWrapper {
    global_pool: CallPool,
    num_pool: CallPool,
    bool_pool: CallPool,
    string_pool: CallPool,
    void_pool: CallPool,
    error_pool: CallPool,
    vec_pool: CallPool,
    map_pool: CallPool,
    executable_pool: CallPool,
    library_pool: CallPool,
    map_pair_pool: CallPool,
    lib_type_pool: CallPool,
    target_properties_pool: CallPool,
    on_off_pool: CallPool,
}

impl CallPoolsWrapper {
    pub(crate) fn new() -> Self {
        Self {
            global_pool: get_global_functions(),
            num_pool: types::get_num_call_pool(),
            string_pool: types::get_string_call_pool(),
            bool_pool: types::get_bool_call_pool(),
            void_pool: types::get_void_call_pool(),
            error_pool: types::get_error_call_pool(),
            vec_pool: types::get_vec_call_pool(),
            map_pool: types::get_map_call_pool(),
            executable_pool: types::get_executable_call_pool(),
            library_pool: types::get_library_call_pool(),
            map_pair_pool: types::get_map_pair_call_pool(),
            lib_type_pool: types::get_lib_type_call_pool(),
            target_properties_pool: types::get_target_properties_call_pool(),
            on_off_pool: types::get_on_off_call_pool(),
        }
    }

    pub(crate) fn get_global_pool(&self) -> &CallPool {
        &self.global_pool
    }

    pub(crate) fn get_num_pool(&self) -> &CallPool {
        &self.num_pool
    }

    pub(crate) fn get_bool_pool(&self) -> &CallPool {
        &self.bool_pool
    }

    pub(crate) fn get_string_pool(&self) -> &CallPool {
        &self.string_pool
    }

    pub(crate) fn get_void_pool(&self) -> &CallPool {
        &self.void_pool
    }

    pub(crate) fn get_error_pool(&self) -> &CallPool {
        &self.error_pool
    }

    pub(crate) fn get_vec_pool(&self) -> &CallPool {
        &self.vec_pool
    }

    pub(crate) fn get_map_pool(&self) -> &CallPool {
        &self.map_pool
    }

    pub(crate) fn get_executable_pool(&self) -> &CallPool {
        &self.executable_pool
    }

    pub(crate) fn get_library_pool(&self) -> &CallPool {
        &self.library_pool
    }

    pub(crate) fn get_map_pair_pool(&self) -> &CallPool {
        &self.map_pair_pool
    }

    pub(crate) fn get_lib_type_pool(&self) -> &CallPool {
        &self.lib_type_pool
    }

    pub(crate) fn get_target_properties_pool(&self) -> &CallPool {
        &self.target_properties_pool
    }

    pub(crate) fn get_on_off_pool(&self) -> &CallPool {
        &self.on_off_pool
    }

    pub(crate) fn get_type_pool(&self, type_: TypeId) -> &CallPool {
        match type_ {
            TypeId::I32 | TypeId::I64 | TypeId::U32 | TypeId::U64 => self.get_num_pool(),
            TypeId::String => self.get_string_pool(),
            TypeId::Void => self.get_void_pool(),
            TypeId::Error => self.get_error_pool(),
            TypeId::Bool => self.get_bool_pool(),
            TypeId::Vec => self.get_vec_pool(),
            TypeId::Map => self.get_map_pool(),
            TypeId::ExecutableReference => self.get_executable_pool(),
            TypeId::LibraryReference => self.get_library_pool(),
            TypeId::MapPair => self.get_map_pair_pool(),
            TypeId::LibType => self.get_lib_type_pool(),
            TypeId::TargetProperties => self.get_target_properties_pool(),
            TypeId::OnOff => self.get_on_off_pool(),
        }
    }
}

pub(crate) struct CallPool {
    executors: Vec<CallExecutor>,
}

impl CallPool {
    pub(crate) fn new(executors: Vec<CallExecutor>) -> Self {
        Self { executors }
    }
}

type ExecutorClosure = dyn Fn(
    Location,
    &AstFuncCallArgs,
    &mut EnvFrame,
    Option<&Value<Box<dyn ValueTypeMarker>>>,
) -> Value<Box<dyn ValueTypeMarker>>;

pub(crate) struct CallExecutor {
    name: String,
    aliases: Vec<String>,
    func: Box<ExecutorClosure>,
}

impl CallExecutor {
    pub(crate) fn new<F>(name: String, func: F, aliases: Vec<String>) -> CallExecutor
    where
        F: 'static
            + Fn(
                Location,
                &AstFuncCallArgs,
                &mut EnvFrame,
                Option<&Value<Box<dyn ValueTypeMarker>>>,
            ) -> Value<Box<dyn ValueTypeMarker>>,
    {
        Self {
            name,
            func: Box::new(func),
            aliases,
        }
    }
}

pub(crate) fn func_call_result(
    call: &AstFuncCall,
    frame: &mut EnvFrame,
) -> Value<Box<dyn ValueTypeMarker>> {
    eval_call(
        call.get_name(),
        call.get_name_loc(),
        call.get_args(),
        frame,
        frame.get_pools_wrapper().get_global_pool(),
        None,
    )
}

pub(crate) fn method_call_result(
    method_property: &AstPropertyAccess,
    call_args: &AstFuncCallArgs,
    frame: &mut EnvFrame,
) -> Value<Box<dyn ValueTypeMarker>> {
    let value = method_property.get_base().eval_in_env(frame);
    eval_call(
        method_property.get_property_name(),
        method_property.get_property_name_loc(),
        call_args,
        frame,
        frame
            .get_pools_wrapper()
            .get_type_pool(value.get_value().get_type_id()),
        Some(&value),
    )
}

pub(crate) fn property_access(
    property: &AstPropertyAccess,
    frame: &mut EnvFrame,
) -> Value<Box<dyn ValueTypeMarker>> {
    let base_expr = property.get_base();
    let base_location = base_expr.get_rng();
    let base = base_expr.eval_in_env(frame);
    let property_name = property.get_property_name();
    match base.get_value().get_type_id() {
        TypeId::I32 | TypeId::I64 | TypeId::U32 | TypeId::U64 => resolve_num_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::String => resolve_str_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::Void => Value::new(Box::new(())),
        TypeId::Error => Value::new(Box::new(types::ErrorValue::new())),
        TypeId::Bool => Value::new(Box::new(())),
        TypeId::Vec => resolve_vec_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::Map => resolve_map_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::ExecutableReference => resolve_executable_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::LibraryReference => resolve_library_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::MapPair => resolve_map_pair_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::LibType => resolve_lib_type_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::TargetProperties => resolve_target_properties_type_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
        TypeId::OnOff => resolve_on_off_type_property_access(
            base,
            base_location,
            property_name,
            property.get_property_name_loc().clone(),
            frame,
        ),
    }
}

include!("interpreter_internal.rs");
mod tasks;
