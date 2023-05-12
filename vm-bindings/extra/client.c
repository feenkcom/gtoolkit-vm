#include <sys/stat.h>
#include "pharovm/pharo.h"
#include "pharovm/pharoClient.h"
#include "pharovm/pathUtilities.h"

extern void setMaxStacksToPrint(sqInt anInteger);
extern sqInt setMaxOldSpaceSize(sqInt anInteger);
extern sqInt setDesiredCogCodeSize(sqInt anInteger);
extern sqInt setDesiredEdenBytes(sqLong anInteger);
extern void setMinimalPermSpaceSize(sqInt min);

#if defined(__GNUC__) && ( defined(i386) || defined(__i386) || defined(__i386__)  \
			|| defined(i486) || defined(__i486) || defined (__i486__) \
			|| defined(intel) || defined(x86) || defined(i86pc) )
static void fldcw(unsigned int cw)
{
    __asm__("fldcw %0" :: "m"(cw));
}
#else
#   define fldcw(cw)
#endif

#if defined(__GNUC__) && ( defined(ppc) || defined(__ppc) || defined(__ppc__)  \
			|| defined(POWERPC) || defined(__POWERPC) || defined (__POWERPC__) )
void mtfsfi(unsigned long long fpscr)
{
    __asm__("lfd   f0, %0" :: "m"(fpscr));
    __asm__("mtfsf 0xff, f0");
}
#else
#   define mtfsfi(fpscr)
#endif

static int loadPharoImage(const char* fileName);

EXPORT(int) vmRunOnWorkerThread = 0;

//TODO: All this should be concentrated in an unique vm parameters structure.
EXPORT(int)
isVMRunOnWorkerThread(void)
{
    return vmRunOnWorkerThread;
}

EXPORT(int) vm_init(VMParameters* parameters)
{
	initGlobalStructure();

	//Unix Initialization specific
	fldcw(0x12bf);	/* signed infinity, round to nearest, REAL8, disable intrs, disable signals */
    mtfsfi(0);		/* disable signals, IEEE mode, round to nearest */

    ioInitTime();

#ifdef PHARO_VM_IN_WORKER_THREAD
    ioVMThread = ioCurrentOSThread();
#endif

	ioInitExternalSemaphores();
	setMaxStacksToPrint(parameters->maxStackFramesToPrint);
	setMaxOldSpaceSize(parameters->maxOldSpaceSize);
    setDesiredEdenBytes(parameters->edenSize);
    setMinimalPermSpaceSize(parameters->minPermSpaceSize);

	if(parameters->maxCodeSize > 0) {
#ifndef COGVM
		logError("StackVM does not accept maxCodeSize");
#else
		logInfo("Setting codeSize to: %ld", parameters->maxCodeSize);
		setDesiredCogCodeSize(parameters->maxCodeSize);
#endif
	}

	aioInit();

	setPharoCommandLineParameters(parameters->vmParameters.parameters, parameters->vmParameters.count,
			parameters->imageParameters.parameters, parameters->imageParameters.count);

	return loadPharoImage(parameters->imageFileName);
}

EXPORT(void)
vm_run_interpreter()
{
	interpret();
}

static int
loadPharoImage(const char* fileName)
{
    struct stat sb;

    /* Check image exists */
    if (stat(fileName, &sb) == -1) {
        logErrorFromErrno("Image file not found");
        return false;
    }

    readImageNamed(fileName);

    char* fullImageName = alloca(FILENAME_MAX);
    fullImageName = getFullPath(fileName, fullImageName, FILENAME_MAX);

    setImageName(fullImageName);

    return 1;
}
