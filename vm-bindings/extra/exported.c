#include "exported.h"

void* exportGetHandler(sqInt anOop) {
    return getHandler(anOop);
}

void* exportReadAddress(sqInt anExternalAddress) {
    return readAddress(anExternalAddress);
}

int exportOsCogStackPageHeadroom() {
    return osCogStackPageHeadroom();
}

VirtualMachine* exportSqGetInterpreterProxy() {
    return sqGetInterpreterProxy();
}

void setVmRunOnWorkerThread(int isOnWorker) {
    vmRunOnWorkerThread = isOnWorker;
}

usqLong exportStatFullGCUsecs() {
    return getStatFullGCUsecs();
}

usqLong exportStatScavengeGCUsecs() {
    return getStatScavengeGCUsecs();
}

sqInt exportClassOrNilAtIndex(sqInt classIndex) {
    return classOrNilAtIndex(classIndex);
}
