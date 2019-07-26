; ModuleID = 'main_module'
source_filename = "main_module"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"

%Person = type { i32, i1 }

@0 = private unnamed_addr constant [20 x i8] c"Age: %5d Alive: %5d\00"

declare i32 @printf(i8*, ...) #0

declare i32 @puts(i8*, ...) #0

define %Person @new_person(i32, i1) {
entry:
  %age = alloca i32
  store i32 %0, i32* %age
  %alive = alloca i1
  store i1 %1, i1* %alive
  ret %Person { i32 3, i1 true }
}

define void @main() {
entry:
  %0 = call %Person @new_person(i32 5, i1 true)
  %p = alloca %Person
  store %Person %0, %Person* %p
  %1 = load %Person, %Person* %p
  %2 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([20 x i8], [20 x i8]* @0, i32 0, i32 0), %Person %1)
  ret void
}

attributes #0 = { "no-frame-pointer-elim"="false" }
