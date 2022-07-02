#include "sq.h"
#include "exportDefinition.h"
#include "sqVirtualMachine.h"

extern void* getHandler(sqInt anOop);
extern void* readAddress(sqInt anExternalAddress);
extern int osCogStackPageHeadroom();
extern sqInt instantiateClassisPinned(sqInt classObj, sqInt isPinned);
extern void * firstBytePointerOfDataObject(sqInt objOop);
extern usqLong getStatFullGCUsecs();
extern usqLong getStatScavengeGCUsecs();

EXPORT(void*) exportGetHandler(sqInt anOop);
EXPORT(void*) exportReadAddress(sqInt anExternalAddress);
EXPORT(int) exportOsCogStackPageHeadroom();
EXPORT(VirtualMachine*) exportSqGetInterpreterProxy();
EXPORT(sqInt) exportInstantiateClassIsPinned(sqInt classObj, sqInt isPinned);
EXPORT(void*) exportFirstBytePointerOfDataObject(sqInt objOop);
EXPORT(usqLong) exportStatFullGCUsecs();
EXPORT(usqLong) exportStatScavengeGCUsecs();

extern int vmRunOnWorkerThread;
EXPORT(void) setVmRunOnWorkerThread(int isOnWorker);