#ifndef TDKG_CORE_H
#define TDKG_CORE_H

// Auto-generated file. Do not edit manually.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

int32_t add(int32_t a, int32_t b);

int32_t multiply(int32_t a, int32_t b);

char *get_version(void);

void free_string(char *ptr);

int32_t calculate_sum(const int32_t *array, uintptr_t length);

char *create_hello_message(const char *name);

#endif  /* TDKG_CORE_H */
