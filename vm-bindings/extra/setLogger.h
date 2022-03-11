#include <stdbool.h>

#include "sq.h"
#include "exportDefinition.h"

EXPORT(void) setLogger(void (*newLogger)(const char* type, const char* fileName, const char* functionName, int line, const char* message));
EXPORT(void) setShouldLog(bool (*newShouldLog)(const char* type));