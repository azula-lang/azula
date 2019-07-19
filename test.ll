; ModuleID = 'main_module'
source_filename = "main_module"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"

@0 = private unnamed_addr constant [6 x i8] c"even!\00"

declare i32 @puts(i8*, ...) #0

define void @main() {
entry:
  %i = alloca i32
  store i32 0, i32* %i
  br label %loop-cond

after:                                            ; preds = %loop-cond
  ret void

loop-cond:                                        ; preds = %entry, %loop
  %0 = load i32, i32* %i
  %1 = icmp slt i32 %0, 10
  br i1 %1, label %loop, label %after

loop:                                             ; preds = %loop-cond
  %2 = load i32, i32* %i
  %3 = sdiv i32 %2, 2
  %4 = mul i32 2, %3
  %5 = sub i32 %3, %4
  %6 = icmp eq i32 %5, 0
  br i1 %6, label %if, label %after1
  br label %loop-cond

after1:                                           ; preds = %loop, %if
  ret void
if:                                               ; preds = %loop
  %7 = call i32 (i8*, ...) @puts(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @0, i32 0, i32 0))
  br label %after1
}

attributes #0 = { "no-frame-pointer-elim"="false" }
