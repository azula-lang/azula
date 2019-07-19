; ModuleID = 'main_module'
source_filename = "main_module"

@0 = private unnamed_addr constant [9 x i8] c"it was 1\00"
@1 = private unnamed_addr constant [11 x i8] c"it was not\00"

declare i32 @printf(i8*, ...)

define i32 @fib(i32) {
entry:
  %x = alloca i32
  store i32 %0, i32* %x
  %1 = load i32, i32* %x
  %2 = icmp eq i32 %1, 0
  br i1 %2, label %if, label %else

after:                                            ; preds = %after1, %if
  ret i32 5

if:                                               ; preds = %entry
  ret i32 0
  br label %after

else:                                             ; preds = %entry
  %4 = load i32, i32* %x
  %5 = icmp eq i32 %4, 1
  br i1 %5, label %if2, label %else3

after1:                                           ; preds = %else3, %if2
  br label %after

if2:                                              ; preds = %else
  ret i32 1
  br label %after1

else3:                                            ; preds = %else
  %7 = load i32, i32* %x
  %8 = sub i32 %7, 1
  %9 = call i32 @fib(i32 %8)
  %10 = load i32, i32* %x
  %11 = sub i32 %7, 2
  %12 = call i32 @fib(i32 %11)
  %13 = add i32 %9, %12
  ret i32 %13
  br label %after1
}

define void @main() {
entry:
  %0 = call i32 @fib(i32 35)
  %x = alloca i32
  store i32 %0, i32* %x
  %1 = load i32, i32* %x
  %2 = icmp eq i32 %1, 9227465
  br i1 %2, label %if, label %else

after:                                            ; preds = %else, %if
  ret void

if:                                               ; preds = %entry
  %3 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @0, i32 0, i32 0))
  br label %after

else:                                             ; preds = %entry
  %4 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @1, i32 0, i32 0))
  br label %after
}
