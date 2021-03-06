# Change Log

## 0.10.0

- Remove lifetime parameter from most of cmdtree aspects.

### 0.10.1
- Update dependencies

## 0.9.0

- Structure and completion aspects now return specific data types with more information such as the item type and the help message.

## 0.8.0

- Feature gated the `runnable` sections, (`run`, `run_with_completion`). Set as default.

## 0.7.0

- Changed acion function trait signature from `Box<Write>` to `&mut Write`
- Added in completion :smile: Look at documentation and examples for use.

## 0.6.0

- Added a `structure()` function on `Commander`

> 0.6.1
> - Added `root_name` to `Commander`

## 0.5.0

- Actions now pass through the writer (from `parse_line`)

## 0.4.0

- Made `Commander` `Send`able, means actions must be `Send`able as well.

> 0.4.2
> - When run interactively, history is now stored.
> 
> 0.4.1
> - Added `action_result` fn to `LineResult`

## 0.3.0

- Added `root()` to `BuilderChain` which short-circuits back to the root.
- Added `at_root()` to `Commander` to flag if currently sitting on root.

## 0.2.0

- Actions now produce a specified return type, defaults to `()`
- `LineResult` has variants pertaining to the outcome of `parse_line`
- `LineResult` is public
