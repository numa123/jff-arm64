// #include <stdio.h>
int main() {
  for (int i = 0; i < 100; i = i + 1) {
    if (i % 3 == 0 && i % 5 == 0) {
      printf("FizzBuzz\n");
    } else if (i % 3 == 0) {
      printf("Fizz\n");
    } else if (i % 5 == 0) {
      printf("Buzz\n");
    } else {
      printf("n\n");
    }
  }
  return 0;
}

/*
good morning!!!!!!!!!!!!
*/