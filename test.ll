; ModuleID = 'main_module'
source_filename = "main_module"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"

@0 = private unnamed_addr constant [8 x i8] c"%f %c%c\00"

declare i32 @printf(i8*, ...) #0

declare i32 @puts(i8*, ...) #0

define void @main() {
entry:
  %my_float = alloca float
  store float 0x402476D5C0000000, float* %my_float
  %0 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([8 x i8], [8 x i8]* @0, i32 0, i32 0), i32 10, i32 13)
  ret void
}

attributes #0 = { "no-frame-pointer-elim"="false" }
