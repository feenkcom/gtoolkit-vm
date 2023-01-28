#ifndef SQ_EXPORT_H_
#define SQ_EXPORT_H_

#include "exportDefinition.h"
#include "pharovm/sqNamedPrims.h"

#define NULL 0

EXPORT(sqExport*) getVMExports();
EXPORT(void) setVMExports(sqExport *exports);

#endif // SQ_EXPORT_H_