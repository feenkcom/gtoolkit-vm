#include "sq.h"
#include "exportDefinition.h"
#include "sqVirtualMachine.h"

extern void* getHandler(sqInt anOop);
extern void* readAddress(sqInt anExternalAddress);
extern int osCogStackPageHeadroom();

extern usqLong getStatFullGCUsecs();
extern usqLong getStatScavengeGCUsecs();
extern sqInt classOrNilAtIndex(sqInt classIndex);

EXPORT(void*) exportGetHandler(sqInt anOop);
EXPORT(void*) exportReadAddress(sqInt anExternalAddress);
EXPORT(int) exportOsCogStackPageHeadroom();
EXPORT(VirtualMachine*) exportSqGetInterpreterProxy();
EXPORT(usqLong) exportStatFullGCUsecs();
EXPORT(usqLong) exportStatScavengeGCUsecs();
EXPORT(sqInt) exportClassOrNilAtIndex(sqInt classIndex);

// Custom
EXPORT(sqInt) createNewMethodheaderbytecodeCount(sqInt class, sqInt header, sqInt bytecodeCount);

// InterpreterPrimitives
EXPORT(sqInt) primitiveFail(void);
EXPORT(sqInt) primitiveFailFor(sqInt code);

// StackInterpreter
EXPORT(sqInt) methodReturnValue(sqInt oop);
EXPORT(sqInt) methodReturnBool(sqInt boolean);
EXPORT(sqInt) methodReturnFloat(double aFloat);
EXPORT(sqInt) methodReturnInteger(sqInt integer);
EXPORT(sqInt) methodReturnReceiver(void);
EXPORT(sqInt) methodArgumentCount(void);
EXPORT(sqInt) stackValue(sqInt offset);
EXPORT(double) stackFloatValue(sqInt offset);
EXPORT(sqInt) stackIntegerValue(sqInt offset);
EXPORT(sqInt) stackObjectValue(sqInt offset);
EXPORT(sqInt) stObjectat(sqInt array, sqInt index);
EXPORT(sqInt) stObjectatput(sqInt array, sqInt index, sqInt value);
EXPORT(sqInt) stSizeOf(sqInt oop);
EXPORT(sqInt) addressCouldBeClassObj(sqInt oop);
EXPORT(sqInt) isKindOfClass(sqInt oop, sqInt aClass);
EXPORT(sqInt) getThisContext(void);

// CoInterpreter
EXPORT(sqInt) instVarofContext(sqInt offset, sqInt oop);

// SpurMemoryManager
EXPORT(sqInt) falseObject();
EXPORT(sqInt) trueObject();
EXPORT(sqInt) nilObject();
EXPORT(sqInt) classArray();
EXPORT(sqInt) classExternalAddress();
EXPORT(sqInt) classString();
EXPORT(void *) firstIndexableField(sqInt oop);
EXPORT(void *) firstFixedField(sqInt oop);
EXPORT(sqInt) instantiateClassindexableSize(sqInt classObj, sqInt nElements);
EXPORT(sqInt) instantiateClassindexableSizeisPinned(sqInt classObj, sqInt nElements, sqInt isPinned);
EXPORT(sqInt) instantiateClassisPinned(sqInt classObj, sqInt isPinned);
EXPORT(void) possibleOldObjectStoreInto(sqInt destObj);
EXPORT(void) possiblePermObjectStoreIntovalue(sqInt destObj, sqInt valueObj);
EXPORT(sqInt) fetchPointerofObject(sqInt fieldIndex, sqInt objOop);
EXPORT(sqInt) integerObjectOf(sqInt value);
EXPORT(sqInt) floatObjectOf(double aFloat);
EXPORT(double) floatValueOf(sqInt objOop);
EXPORT(sqInt) isFloatInstance(sqInt objOop);
EXPORT(sqInt) newHashBitsOf(sqInt objOop);
EXPORT(sqInt) hashBitsOf(sqInt objOop);
EXPORT(sqInt) ensureBehaviorHash(sqInt objOop);
EXPORT(void *) firstBytePointerOfDataObject(sqInt objOop);
EXPORT(sqInt) isOopForwarded(sqInt oop);
EXPORT(sqInt) isOld(sqInt oop);
EXPORT(sqInt) isYoung(sqInt oop);
EXPORT(sqInt) fetchClassOfNonImm(sqInt oop);
EXPORT(sqInt) stContextSize(sqInt oop);

extern int vmRunOnWorkerThread;
EXPORT(void) setVmRunOnWorkerThread(int isOnWorker);