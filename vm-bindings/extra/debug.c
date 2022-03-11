#include <stdarg.h>
#include <stdbool.h>
#include "pharovm/pharo.h"
#include "setLogger.h"
#include "debug.h"
#include "beacon.h"

#ifdef _WIN32

#else
#include <sys/time.h>
#endif

#ifndef PATH_MAX
#define PATH_MAX MAX_PATH
#endif

char * GetAttributeString(sqInt id);

void (*logger)(const char* type, const char* fileName, const char* functionName, int line, const char* message);
bool (*shouldLog)(const char* type);

void logLevel(int level) {}

int getLogLevel() {
	return LOG_TRACE;
}

int isLogDebug() {
	return false;
}

void error(char *errorMessage){
    logError(errorMessage);
	printStatusAfterError();
    abort();
}

static const char* severityName[6] = { "NONE", "ERROR", "WARNING", "INFO", "DEBUG", "TRACE" };

void logAssert(const char* fileName, const char* functionName, int line, char* msg){
	logMessage(LOG_WARN, fileName, functionName, line, msg);
}

void logMessageFromErrno(int level, const char* msg, const char* fileName, const char* functionName, int line){
	char buffer[1024+1];

#ifdef WIN32
	strerror_s(buffer, 1024, errno);
#else
	strerror_r(errno, buffer, 1024);
#endif

	logMessage(level, fileName, functionName, line, "%s: %s", msg, buffer);
}

static void logTypedMessage_impl(const char* type, const char* fileName, const char* functionName, int line, va_list args) {
    char * format;
    char * buffer;
    int max_buffer_len;

    if (!shouldLog) {
        return;
    }
    if (!logger) {
    	return;
    }
    if (!shouldLog(type)) {
        return;
    }

    format = va_arg(args, char*);
    max_buffer_len = 250;
    buffer = malloc(max_buffer_len);

    vsnprintf(buffer, max_buffer_len, format, args);

    logger(type, fileName, functionName, line, buffer);

    free(buffer);
}

void setLogger(void (*newLogger)(const char* type, const char* fileName, const char* functionName, int line, const char* message)) {
    logger = newLogger;
}

void setShouldLog(bool (*newShouldLog)(const char* type)) {
    shouldLog = newShouldLog;
}

void logTypedMessage(const char* type, const char* fileName, const char* functionName, int line, ...) {
    va_list args;
    va_start(args, line);
    logTypedMessage_impl(type, fileName, functionName, line, args);
    va_end(args);
}

void logMessage(int level, const char* fileName, const char* functionName, int line, ...) {
    va_list args;
    const char * type;

	va_start(args, line);
	type = severityName[level];

	logTypedMessage_impl(type, fileName, functionName, line, args);

	va_end(args);
}

void getCrashDumpFilenameInto(char *buf)
{
#ifdef _WIN32
	/*
	* Unsafe version of deprecated strcpy for compatibility
	* - does not check error code
	* - does use count as the size of the destination buffer
	*/
	strcat_s(buf, PATH_MAX + 1, "crash.dmp");
#else
	strcat(buf, "crash.dmp");
#endif
}

char *getVersionInfo(int verbose)
{
#if STACKVM
  extern char *__interpBuildInfo;
# define INTERP_BUILD __interpBuildInfo
# if COGVM
  extern char *__cogitBuildInfo;
# endif
#else
# define INTERP_BUILD interpreterVersion
#endif
  extern char *revisionAsString();

#define BUFFER_SIZE 4096

  char *info= (char *)malloc(BUFFER_SIZE);
  info[0]= '\0';

#if SPURVM
# if BytesPerOop == 8
#	define ObjectMemory " Spur 64-bit"
# else
#	define ObjectMemory " Spur"
# endif
#else
# define ObjectMemory
#endif
#if defined(NDEBUG)
# define BuildVariant "Production" ObjectMemory
#elif DEBUGVM
# define BuildVariant "Debug" ObjectMemory
# else
# define BuildVariant "Assert" ObjectMemory
#endif

#if USE_XSHM
#define USE_XSHM_STRING " XShm"
#else
#define USE_XSHM_STRING ""
#endif

#if ITIMER_HEARTBEAT
# define HBID " ITHB"
#else
# define HBID
#endif

  if(verbose){
	  snprintf(info, BUFFER_SIZE, IMAGE_DIALECT_NAME "VM version:" VM_VERSION "-" VM_BUILD_STRING USE_XSHM_STRING " " COMPILER_VERSION " [" BuildVariant HBID " VM]\nBuilt from: %s\n With:%s\n Revision: " VM_BUILD_SOURCE_STRING, INTERP_BUILD, GetAttributeString(1008));
  }else{
	  snprintf(info, BUFFER_SIZE, VM_VERSION "-" VM_BUILD_STRING USE_XSHM_STRING " " COMPILER_VERSION " [" BuildVariant HBID " VM]\n%s\n%s\n" VM_BUILD_SOURCE_STRING, INTERP_BUILD, GetAttributeString(1008));
  }

  return info;
}

/***
 *  This SHOULD be rewritten passing the FILE* as a parameter.
 */

static FILE* outputStream = NULL;

void
vm_setVMOutputStream(FILE * stream){
	fflush(outputStream);
	outputStream = stream;
}

int
vm_printf(const char * format, ... ){

	va_list list;
	va_start(list, format);

	if(outputStream == NULL){
		outputStream = stdout;
	}

	int returnValue = vfprintf(outputStream, format, list);

	va_end(list);

	return returnValue;
}
