// // fizzbuzz

// #include <stdio.h>
// int main() {
//   for (int i = 0; i < 10; i++) {
//     if (i % 3 == 0 && i % 5 == 0) {
//       printf("FizzBuzz\n");
//     } else if (i % 3 == 0) {
//       printf("Fizz\n");
//     } else if (i % 5 == 0) {
//       printf("Buzz\n");
//     } else {
//       printf("%d\n", i);
//     }
//   }
// }

#include <stdio.h>
// int main() {
//   long a = 2 & 2;
//   printf("%d\n", a);
// }

// int main() {
//   for (int i = 1; i < 10; i = i + 1) {
//     if (i % 3 == 0 && i % 2 == 0) {
//       return i;
//     }
//   }
// }

int main() {
  for (int i = 0; i < 100; i = i + 1) {
    if (i % 3 == 0 && i % 5 == 0) {
      printf("FizzBuzz\n");
    } else if (i % 3 == 0) {
      printf("Fizz\n");
    } else if (i % 5 == 0) {
      printf("Buzz\n");
    } else {
      printf("%d\n", i);
    }
  }
  return 0;
}