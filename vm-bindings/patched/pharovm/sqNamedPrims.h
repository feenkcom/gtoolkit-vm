typedef struct {
  char *pluginName;
  char *primitiveName; /* N.B. On Spur the accessorDepth is hidden after this */
  void *primitiveAddress;
} sqExport;

extern sqExport vm_exports[];
extern sqExport os_exports[];
extern sqExport *pluginExports[3] = {
	vm_exports,
	os_exports,
//	SecurityPlugin_exports,
	NULL
};