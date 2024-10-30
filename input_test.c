#include <stdio.h>
#include <stdlib.h>

int main() {
    char *a = malloc(sizeof(char) * 100);
    scanf("%s", a);
    printf("%s\n", a);
    scanf("%s", a);
    printf("%s\n", a);
}
