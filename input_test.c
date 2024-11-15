#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    char *a = malloc(sizeof(char) * 100);
    while (1) {
        fgets(a, 100, stdin);
        printf("%s", a);
        if (strcmp(a, "q\n") == 0) {
            break;
        }
    }
    if (1) {
        printf("%s", a);
    } else
}
