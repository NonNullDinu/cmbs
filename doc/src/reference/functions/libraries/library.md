# The `library()` function

## Parameters
```leafbuild
library (
        library_name,
        files,
)
```

## Description
Creates a library in the current module and returns it.

## Returns
This function returns a `library` object. The type is described [here](../../special_types/library.md).

## Positional parameters

### library_name
> **Type**: `string`

#### Description

The name of the library.
Should not contain `/`s.

### files
> **Type**: `string` or array of `string`s.

#### Description
The list of source files to build the library with.

## Aliases
`lib()`