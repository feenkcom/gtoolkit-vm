#include "sqExport.h"

sqExport* getVMExports()
{
    return pluginExports[0];
}

void setVMExports(sqExport *exports)
{
    pluginExports[0] = exports;
}