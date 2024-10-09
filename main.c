#include "test.h"
#include <stdio.h>
#include <stdlib.h>
void assert(int expected, int actual, char *code) {
  if (expected == actual) {
    printf("%s => %d\n", code, actual);
  } else {
    printf("%s => %d expected but got %d\n", code, expected, actual);
    exit(1);
  }
}
#define ASSERT(x, y) assert(x, y, #y)

int main() {
  ASSERT(1, ({
           char x = 1;
           char y = 2;
           x;
         }));
  printf("OK\n");
  return 0;
}