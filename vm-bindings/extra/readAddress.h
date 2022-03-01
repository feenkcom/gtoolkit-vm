#include "sq.h"
#include "exportDefinition.h"

extern void* readAddress(sqInt anExternalAddress);
EXPORT(void*) exportReadAddress(sqInt anExternalAddress);