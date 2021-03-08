//! # The parser code
//! Also see [the syntax reference](https://leafbuild.github.io/leafbuild/dev/syntax_ref.html).

use std::fmt;
use std::ops::Range;

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, TextRange};

use crate::lexer::Lexer;
use crate::syntax_kind::SyntaxKind::{self, *};
use leafbuild_core::utils::TakeIfUnless;

///
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Span {
    text_range: TextRange,
}

impl From<Range<u32>> for Span {
    fn from(range: Range<u32>) -> Self {
        Self {
            text_range: TextRange::new(range.start.into(), range.end.into()),
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.text_range)
    }
}

///
#[derive(Debug)]
pub struct Parse {
    /// the node
    pub green_node: GreenNode,
    /// errors
    #[allow(unused)]
    pub errors: Vec<(String, Span)>,
}

struct Parser<'input> {
    /// tokens
    tokens: Vec<(SyntaxKind, &'input str, Span)>,
    meaningful: Vec<(SyntaxKind, usize)>,
    index: usize,
    meaningful_index: usize,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<(String, Span)>,
}

/// `is` helper
pub(crate) trait Is: Sized + Copy {
    fn is(self, kind: SyntaxKind) -> bool;

    fn isnt(self, kind: SyntaxKind) -> bool {
        !self.is(kind)
    }

    fn is_any(self, kinds: &[SyntaxKind]) -> bool {
        kinds.iter().any(|&it| self.is(it))
    }
}

impl Is for SyntaxKind {
    fn is(self, kind: SyntaxKind) -> bool {
        self == kind
    }
}

impl Is for Option<SyntaxKind> {
    fn is(self, kind: SyntaxKind) -> bool {
        self.map_or(false, |it| it.is(kind))
    }

    fn isnt(self, kind: SyntaxKind) -> bool {
        self.map_or(false, |it| it.isnt(kind))
    }
}

#[derive(Debug, Clone)]
enum ParseError {
    Eof,
    Incomplete,
    Error(String, Span),
    UnexpectedToken(String, Span),
    ExpectedToken(String, Span, String),
    ExpectedTokens(Vec<String>, Span),
}

trait MapIncomplete {
    /// Eof => Incomplete
    fn map_incomplete(self) -> Self;
}

impl MapIncomplete for ParseError {
    fn map_incomplete(self) -> Self {
        match self {
            Self::Eof => Self::Incomplete,
            other => other,
        }
    }
}

impl<T, E: MapIncomplete> MapIncomplete for Result<T, E> {
    fn map_incomplete(self) -> Self {
        self.map_err(MapIncomplete::map_incomplete)
    }
}

trait Trivia: Copy {
    fn is_trivia(self) -> bool;
    fn is_newline(self) -> bool;

    fn is_meaningful(self) -> bool;
}

#[allow(clippy::inline_always)]
impl<T: Is> Trivia for T {
    #[inline(always)]
    fn is_trivia(self) -> bool {
        self.is_any(&[WHITESPACE, LINE_COMMENT, BLOCK_COMMENT])
    }

    #[inline(always)]
    fn is_newline(self) -> bool {
        self.is(NEWLINE)
    }

    #[inline(always)]
    fn is_meaningful(self) -> bool {
        !self.is_trivia() && !self.is_newline()
    }
}

type ParseResult<T = ()> = std::result::Result<T, ParseError>;

#[allow(clippy::inline_always)]
impl<'input> Parser<'input> {
    fn parse(mut self) -> Parse {
        self.parse_node(ROOT, |p| {
            loop {
                match parse_lang_item(p) {
                    Err(ParseError::Eof) => break,
                    Ok(()) => {}
                    Err(ParseError::Incomplete) => {
                        p.errors.push(("incomplete".into(), Span::default()))
                    }
                    Err(ParseError::Error(err, span)) => {
                        p.errors.push((err, span));
                        break;
                    }
                    Err(ParseError::UnexpectedToken(tk, span)) => {
                        p.errors.push((format!("unexpected `{}`", tk), span));
                        break;
                    }
                    Err(ParseError::ExpectedToken(tk, span, found)) => {
                        p.errors.push((
                            format!("expected token {}, found token {}", tk, found),
                            span,
                        ));
                        break;
                    }
                    Err(ParseError::ExpectedTokens(tokens, span)) => {
                        p.errors
                            .push((format!("expected one of {{{}}}", tokens.join(", ")), span));
                        break;
                    }
                }
            }

            Ok(())
        })
        .unwrap();

        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    /// Advance one meaningful token, adding it to the current branch of the tree builder,
    /// along with all the trivia before it.
    #[inline(always)]
    fn bump(&mut self) {
        self.meaningful_index += 1;
        if let Some(index) = self
            .meaningful
            .get(self.meaningful_index)
            .map(|&(_, it)| it)
        {
            self.bump_raw_to(index);
        }
    }

    #[inline(always)]
    fn bump_last(&mut self) {
        if let Some(index) = self
            .meaningful
            .get(self.meaningful_index)
            .map(|&(_, it)| it)
        {
            self.bump_raw_to(index + 1);
        }
        self.meaningful_index += 1;
    }

    #[inline(always)]
    fn bump_raw(&mut self) {
        if let Some((kind, text, _)) = self.tokens.get(self.index) {
            if self.index
                == self
                    .meaningful
                    .get(self.meaningful_index + 1)
                    .map_or(usize::MAX, |&(_, index)| index)
            {
                self.meaningful_index += 1;
            }

            self.builder.token(kind.into(), text);
            self.index += 1;
        }
    }

    #[inline(always)]
    fn bump_raw_to(&mut self, new_index: usize) {
        let Parser {
            ref index,
            ref tokens,
            ref mut builder,
            ..
        } = self;

        tokens[*index..new_index]
            .iter()
            .for_each(|(kind, text, _)| {
                builder.token(kind.into(), text);
            });

        self.index = new_index;
    }

    #[inline(always)]
    fn bump_if(&mut self, f: impl FnOnce(SyntaxKind) -> bool) -> bool {
        if self.current().map_or(false, f) {
            self.bump();
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn current(&self) -> Option<SyntaxKind> {
        self.meaningful
            .get(self.meaningful_index)
            .map(|(kind, _)| *kind)
    }

    #[inline(always)]
    fn current_span(&self) -> Span {
        self.meaningful
            .get(self.meaningful_index)
            .map(|&(_, index)| index)
            .map_or(Span::default(), |index| self.tokens[index].2)
    }

    #[inline(always)]
    fn current_raw(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.index).map(|(kind, _, _)| *kind)
    }

    #[inline(always)]
    fn current_span_raw(&self) -> Span {
        self.tokens
            .get(self.index)
            .map_or(Span::default(), |(_, _, span)| *span)
    }

    #[inline(always)]
    fn skip_trivia_and_single_newline(&mut self) -> ParseResult {
        self.require_newline()
    }

    #[inline(always)]
    fn next_nontrivia(&self) -> Option<SyntaxKind> {
        self.tokens[self.index..]
            .iter()
            .find_map(|&(it, _, _)| it.take_unless(|&it| it.is_trivia()))
    }

    #[inline(always)]
    fn require_newline(&mut self) -> ParseResult {
        while self.current_raw().is_trivia() {
            self.bump_raw();
        }

        match self.current_raw() {
            Some(NEWLINE) => {
                self.bump_raw();
                Ok(())
            }
            Some(other) => other.as_unexpected_token(self.current_span_raw()),
            None => Err(ParseError::Eof),
        }
    }

    #[inline(always)]
    fn next_nontrivia_lookahead(&self) -> Option<SyntaxKind> {
        self.tokens[self.index + 1..]
            .iter()
            .find_map(|(it, _, _)| (*it).take_unless(|&it| it.is_trivia()))
    }

    fn error(&mut self) {
        self.builder.token(ERROR.into(), "")
    }

    fn parse_single_tok_wrapped(
        &mut self,
        kind: SyntaxKind,
        output_kind: SyntaxKind,
    ) -> ParseResult {
        self.builder.start_node(output_kind.into());
        if !self.bump_if(|it| it.is(kind)) {
            let current = self.current();
            self.errors.push((
                format!("Expected {:?}, got {:?}", kind, current),
                self.current_span(),
            ));
            self.error();
            return self
                .current()
                .unwrap()
                .as_unexpected_token(self.current_span());
        }
        self.builder.finish_node();
        Ok(())
    }

    fn parse_single_tok(&mut self, kind: SyntaxKind) -> ParseResult {
        if !self.bump_if(|it| it.is(kind)) {
            self.error();

            return Err(ParseError::ExpectedToken(
                kind.token_name(),
                self.current_span(),
                self.current().unwrap_or(ERROR).token_name(),
            ));
        }
        Ok(())
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into())
    }

    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into())
    }

    fn checkpoint(&mut self) -> Checkpoint {
        self.builder.checkpoint()
    }

    fn finish_node(&mut self) {
        self.builder.finish_node()
    }

    #[inline(always)]
    fn parse_node<T>(
        &mut self,
        kind: SyntaxKind,
        f: impl FnOnce(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        self.start_node(kind);
        let r = f(self);
        self.finish_node();

        r
    }

    #[inline(always)]
    fn parse_node_at<T>(
        &mut self,
        checkpoint: Checkpoint,
        kind: SyntaxKind,
        f: impl FnOnce(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        self.start_node_at(checkpoint, kind);
        let r = f(self);
        self.finish_node();

        r
    }
}

/// parses `text`
#[must_use]
pub fn parse(text: &str) -> Parse {
    let lexer = Lexer::new(text);
    let tokens: Vec<(SyntaxKind, &str, Span)> = lexer.collect();
    let meaningful = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, (it, _, _))| (*it, index).take_if(|(kind, _)| kind.is_meaningful()))
        .collect();
    Parser {
        builder: GreenNodeBuilder::new(),
        errors: vec![],
        index: 0,
        tokens,
        meaningful,
        meaningful_index: 0,
    }
    .parse()
}

#[inline]
fn parse_lang_item(p: &mut Parser) -> ParseResult {
    parse_statement(p)?;

    Ok(())
}

trait AsUnexpectedToken: Copy {
    fn as_unexpected_token(self, span: Span) -> ParseResult;
}

impl AsUnexpectedToken for SyntaxKind {
    fn as_unexpected_token(self, span: Span) -> ParseResult {
        Err(ParseError::UnexpectedToken(self.token_name(), span))
    }
}

fn parse_statement(p: &mut Parser) -> ParseResult {
    let tok = p.current().ok_or(ParseError::Eof)?;
    match tok {
        L_PAREN | L_BRACKET | L_BRACE | PLUS | MINUS | NOT_KW | TRUE_KW | FALSE_KW | NUMBER
        | ID | STRING | MULTILINE_STRING => {
            let assignment_checkpoint = p.checkpoint();
            parse_expr(p)?;
            if p.current()
                .is_any(&[PLUS_EQ, MINUS_EQ, MUL_EQ, DIV_EQ, MOD_EQ, EQ])
            {
                p.parse_node_at(assignment_checkpoint, Assignment, |p| {
                    // consume the `=` / `+=` / ...
                    p.bump();

                    parse_expr(p)?;

                    p.require_newline()?;
                    Ok(())
                })
            } else {
                Ok(())
            }
        }
        R_PAREN | R_BRACKET | R_BRACE | PLUS_EQ | MINUS_EQ | MUL_EQ | DIV_EQ | MOD_EQ
        | ASTERISK | SLASH | PERCENT | EQ_EQ | GREATER_EQ | GREATER | LESS_EQ | LESS | NOT_EQ
        | EQ | SHIFT_LEFT | SHIFT_RIGHT | DOT | COLON | QMARK | SEMICOLON | COMMA | TILDE
        | AND_KW | OR_KW | IN_KW | FN_KW | ELSE_KW | ERROR => {
            tok.as_unexpected_token(p.current_span())
        }
        LET_KW => parse_declaration(p),
        IF_KW => parse_conditional(p),
        FOREACH_KW => parse_foreach(p),
        CONTINUE_KW | BREAK_KW | RETURN_KW => parse_control_stmt(p),
        _ => unreachable!(),
    }
}

fn parse_expr(p: &mut Parser) -> ParseResult {
    p.parse_node(Expr, parse_precedence_9_expr)
}

fn parse_tuple_expr(p: &mut Parser) -> ParseResult {
    assert!(is_tuple_expr_start(p));

    parse_tt(p, TupleExpr, L_PAREN, Some(COMMA), R_PAREN, parse_expr)
}

fn is_tuple_expr_start(p: &mut Parser) -> bool {
    p.current().is(L_PAREN)
}

fn parse_array_expr(p: &mut Parser) -> ParseResult {
    assert!(is_array_expr_start(p));

    parse_tt(
        p,
        ArrayLitExpr,
        L_BRACKET,
        Some(COMMA),
        R_BRACKET,
        parse_expr,
    )
}

fn is_array_expr_start(p: &mut Parser) -> bool {
    p.current().is(L_BRACKET)
}

fn parse_primary(p: &mut Parser) -> ParseResult {
    p.parse_node(PrimaryExpr, |p| {
        if is_array_expr_start(p) {
            parse_array_expr(p)
        } else if is_tuple_expr_start(p) {
            parse_tuple_expr(p)
        } else if is_conditional_start(p) {
            parse_conditional(p)
        } else if is_expr_block_start(p) {
            parse_expr_block(p)
        } else if p.current().is_any(&[NUMBER, ID]) {
            p.bump_last();
            Ok(())
        } else if is_string_lit(p) {
            parse_string(p)
        } else {
            p.error();
            p.current().map_or(Err(ParseError::Eof), |token| {
                token.as_unexpected_token(p.current_span())
            })
        }
    })
}

fn is_string_lit(p: &mut Parser) -> bool {
    p.current().is_any(&[STRING, MULTILINE_STRING])
}

fn parse_string(p: &mut Parser) -> ParseResult {
    assert!(is_string_lit(p));
    p.parse_node(StrLit, |p| {
        p.bump_last();
        Ok(())
    })
}

fn parse_tt(
    p: &mut Parser,
    outer_kind: SyntaxKind,
    start_tok: SyntaxKind,
    separator: Option<SyntaxKind>,
    end_tok: SyntaxKind,
    mut f: impl FnMut(&mut Parser) -> ParseResult,
) -> ParseResult {
    assert!(p.current().is(start_tok));
    p.parse_node(outer_kind, move |p| {
        p.bump();

        while p.current().isnt(end_tok) {
            f(p).map_incomplete()?;

            if let Some(separator) = separator {
                if !p.bump_if(|it| it.is(separator)) && p.current().isnt(end_tok) {
                    p.error();

                    return Err(ParseError::ExpectedTokens(
                        vec![end_tok.token_name(), separator.token_name()],
                        p.current_span(),
                    ));
                }
            }
        }

        // consume the end token
        p.bump_last();

        Ok(())
    })
}

fn parse_precedence_1_expr(p: &mut Parser) -> ParseResult {
    let ck = p.checkpoint();
    parse_primary(p)?;

    while p.current().is_any(&[L_PAREN, L_BRACKET]) {
        if p.current().is(L_PAREN) {
            parse_f_call(p, ck)?
        } else if p.current().is(L_BRACKET) {
            parse_index_expr(p, ck)?
        }
    }

    Ok(())
}

fn parse_f_call(p: &mut Parser, ck: Checkpoint) -> ParseResult {
    p.parse_node_at(ck, FuncCallExpr, |p| {
        parse_tt(p, FuncCallArgs, L_PAREN, Some(COMMA), R_PAREN, parse_farg)
    })
}

fn parse_farg(p: &mut Parser) -> ParseResult {
    if is_kexpr_start(p) {
        parse_kexpr(p)
    } else {
        parse_expr(p)
    }
}

fn parse_kexpr(p: &mut Parser) -> ParseResult {
    assert!(is_kexpr_start(p));

    p.parse_node(KExpr, |p| {
        p.bump();

        p.parse_single_tok(EQ)?;

        parse_expr(p)?;

        Ok(())
    })
}

fn is_kexpr_start(p: &mut Parser) -> bool {
    p.current().is(ID) && p.next_nontrivia_lookahead().is(EQ)
}

fn parse_index_expr(p: &mut Parser, ck: Checkpoint) -> ParseResult {
    assert!(p.current().is(L_BRACKET));
    p.parse_node_at(ck, IndexedExpr, |p| {
        p.parse_node(IndexedExprBrackets, |p| {
            p.bump(); // '['
            parse_expr(p)?; // expr
            p.parse_single_tok(R_BRACKET)?;

            Ok(())
        })
    })
}

fn parse_precedence_2_expr(p: &mut Parser) -> ParseResult {
    if p.current().is_any(&[PLUS, MINUS]) {
        p.parse_node(PrefixUnaryOpExpr, |p| {
            p.bump();

            parse_precedence_2_expr(p)
        })
    } else {
        parse_precedence_1_expr(p)
    }
}

fn parse_infix_binop(
    p: &mut Parser,
    ops: &[SyntaxKind],
    mut lower: impl FnMut(&mut Parser) -> ParseResult,
) -> ParseResult {
    let ck = p.checkpoint();
    lower(p)?;

    while p.current().is_any(ops) {
        p.parse_node_at(ck, InfixBinOpExpr, |p| {
            p.bump();
            lower(p)?;

            Ok(())
        })?;
    }

    Ok(())
}

fn parse_precedence_3_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[ASTERISK, SLASH, PERCENT], parse_precedence_2_expr)
}

fn parse_precedence_4_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[PLUS, MINUS], parse_precedence_3_expr)
}

fn parse_precedence_5_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[SHIFT_LEFT, SHIFT_RIGHT], parse_precedence_4_expr)
}

fn parse_precedence_6_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(
        p,
        &[LESS, LESS_EQ, GREATER, GREATER_EQ],
        parse_precedence_5_expr,
    )
}

fn parse_precedence_7_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[EQ_EQ, NOT_EQ], parse_precedence_6_expr)
}

fn parse_precedence_8_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[AND_KW], parse_precedence_7_expr)
}

fn parse_precedence_9_expr(p: &mut Parser) -> ParseResult {
    parse_infix_binop(p, &[OR_KW], parse_precedence_8_expr)
}

fn parse_expr_block(p: &mut Parser) -> ParseResult {
    parse_tt(p, ExprBlock, L_BRACE, None, R_BRACE, parse_statement)
}

fn is_expr_block_start(p: &mut Parser) -> bool {
    p.current().is(L_BRACE)
}

fn parse_declaration(p: &mut Parser) -> ParseResult {
    assert!(p.current().is(LET_KW));
    p.parse_node(Declaration, |p| {
        p.bump(); // LET_KW

        p.parse_single_tok(ID).map_incomplete()?;

        p.parse_single_tok(EQ).map_incomplete()?;

        parse_expr(p).map_incomplete()?;

        p.require_newline().map_incomplete()?;

        Ok(())
    })
}

fn is_conditional_start(p: &mut Parser) -> bool {
    p.current().is(IF_KW)
}

fn parse_conditional(p: &mut Parser) -> ParseResult {
    assert!(is_conditional_start(p));

    p.parse_node(Conditional, |p| {
        parse_conditional_branch(p).map_incomplete()?;
        while p.current().is(ELSE_KW) {
            p.bump();
            if p.current().is(IF_KW) {
                parse_conditional_branch(p).map_incomplete()?;
            } else {
                parse_expr_block(p).map_incomplete()?;
                break;
            }
        }

        Ok(())
    })
}

fn parse_conditional_branch(p: &mut Parser) -> ParseResult {
    assert!(p.current().is(IF_KW));
    p.parse_node(ConditionalBranch, |p| {
        // consume the IF_KW
        p.bump();

        parse_expr(p).map_incomplete()?;

        parse_expr_block(p).map_incomplete()?;

        Ok(())
    })
}

fn parse_foreach(p: &mut Parser) -> ParseResult {
    assert!(p.current().is(FOREACH_KW));
    p.parse_node(Foreach, |p| {
        p.bump(); // FOREACH_KW
        parse_expr(p).map_incomplete()?;
        p.parse_single_tok(IN_KW)?;
        parse_expr(p).map_incomplete()?;
        parse_expr_block(p).map_incomplete()?;
        Ok(())
    })
}

fn parse_control_stmt(p: &mut Parser) -> ParseResult {
    p.parse_node(ControlStatement, |p| match p.current() {
        Some(CONTINUE_KW) => {
            p.bump();
            p.skip_trivia_and_single_newline()?;
            Ok(())
        }

        Some(RETURN_KW) | Some(BREAK_KW) => {
            p.bump();
            if p.next_nontrivia().isnt(NEWLINE) {
                parse_expr(p)?;
            }

            p.skip_trivia_and_single_newline()?;

            Ok(())
        }

        Some(thing) => thing.as_unexpected_token(p.current_span()),
        None => Err(ParseError::Incomplete),
    })
}
