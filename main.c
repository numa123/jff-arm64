
long main() {
  long x[2];
  *x = 3;
  *(x + 1) = 4;
  return *(x + 1);
}