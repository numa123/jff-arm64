long main() {
  long x[3];
  *x = 3;
  *(x + 1) = 4;
  *(x + 2) = 5;
  return *x;
}