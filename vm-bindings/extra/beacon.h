#ifndef BEACON_H_
#define BEACON_H_

#include "exportDefinition.h"

EXPORT(void) logTypedMessage(const char* type, const char* fileName, const char* functionName, int line, ...);

#define __FILENAME__ ((char*)__FILE__ + SOURCE_PATH_SIZE)
#define logBeacon(type, ...) logTypedMessage(type, __FILENAME__, __FUNCTION__, __LINE__, __VA_ARGS__)

#endif // BEACON_H_