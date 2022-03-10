#include "sq.h"
#include "exportDefinition.h"

EXPORT(void) setLogger(void (*newLogger)(int level, const char* fileName, const char* functionName, int line, const char* message));
EXPORT(int) exportGetLogLevel();