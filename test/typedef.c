#include "test.h"
int main() {
  ASSERT(1, ({
           typedef int t;
           t x = 1;
           x;
         }));
  ASSERT(1, ({
           typedef struct {
             int a;
           } t;
           t x;
           x.a = 1;
           x.a;
         }));
  ASSERT(1, ({
           typedef int t;
           t t = 1;
           t;
         }));
  ASSERT(2, ({
           typedef struct {
             int a;
           } t;
           { typedef int t; }
           t x;
           x.a = 2;
           x.a;
         }));
  ASSERT(4, ({
           typedef t;
           t x;
           sizeof(x);
         }));
  printf("OK\n");
  return 0;
}