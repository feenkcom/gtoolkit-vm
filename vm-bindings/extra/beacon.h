EXPORT(void) logTypedMessage(const char* type, const char* fileName, const char* functionName, int line, ...);

#define logBeacon(type, ...) logTypedMessage(type, __FILENAME__, __FUNCTION__, __LINE__, __VA_ARGS__)