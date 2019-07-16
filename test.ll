; ModuleID = 'main_module'
source_filename = "main_module"

@yes = global [5 x i8] c"heyo\00"

declare i32 @printf(i8*, ...)

define void @main(i8*, ...) {
entry:
}
