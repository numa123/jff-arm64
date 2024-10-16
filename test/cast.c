#include "test.h"

int main() {
  // charから他の型へのキャストテスト
  ASSERT(3, ({
           char x = 3;
           (int)x;
         }));
  ASSERT(3, ({
           char x = 3;
           (short)x;
         }));
  ASSERT(3, ({
           char x = 3;
           (long)x;
         }));

  // shortから他の型へのキャストテスト
  ASSERT(42, ({
           short x = 42;
           (char)x;
         }));
  ASSERT(42, ({
           short x = 42;
           (int)x;
         }));
  ASSERT(42, ({
           short x = 42;
           (long)x;
         }));

  // intから他の型へのキャストテスト
  ASSERT(100, ({
           int x = 100;
           (char)x;
         }));
  ASSERT(100, ({
           int x = 100;
           (short)x;
         }));
  ASSERT(100, ({
           int x = 100;
           (long)x;
         }));

  // longから他の型へのキャストテスト
  ASSERT(127, ({
           long x = 127;
           (char)x;
         }));
  ASSERT(32767, ({
           long x = 32767;
           (short)x;
         }));
  ASSERT(2147483647, ({
           long x = 2147483647;
           (int)x;
         }));

  // （符号付きキャスト）
  ASSERT(-128, ({
    char x = -128;
    (int)x;
  }));
  ASSERT(-1, ({
    short x = -1;
    (char)x;
  }));
  ASSERT(-32768, ({
    int x = -32768;
    (short)x;
  }));
  ASSERT(-2147483648, ({
    long x = -2147483648;
    (int)x;
  }));

  printf("OK\n");
  return 0;
}
