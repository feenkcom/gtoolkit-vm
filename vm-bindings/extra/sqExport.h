#include "exportDefinition.h"

#define NULL 0

typedef struct {
  char *pluginName;
  char *primitiveName; /* N.B. On Spur the accessorDepth is hidden after this */
  void *primitiveAddress;
} sqExport;

sqExport *pluginExports[];
EXPORT(sqExport*) getVMExports();
EXPORT(void) setVMExports(sqExport *exports);