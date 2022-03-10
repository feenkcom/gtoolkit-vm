#include "sq.h"
#include "exportDefinition.h"
#include "sqVirtualMachine.h"

extern void* getHandler(sqInt anOop);
extern void* readAddress(sqInt anExternalAddress);
extern int osCogStackPageHeadroom();
extern sqInt instantiateClassisPinned(sqInt classObj, sqInt isPinned);

EXPORT(void*) exportGetHandler(sqInt anOop);
EXPORT(void*) exportReadAddress(sqInt anExternalAddress);
EXPORT(int) exportOsCogStackPageHeadroom();
EXPORT(VirtualMachine*) exportSqGetInterpreterProxy();
EXPORT(sqInt) exportInstantiateClassIsPinned(sqInt classObj, sqInt isPinned);

extern int vmRunOnWorkerThread;
EXPORT(void) setVmRunOnWorkerThread(int isOnWorker);