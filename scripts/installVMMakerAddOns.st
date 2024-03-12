Author useAuthor: 'installVMMaker' during: [
	path := CommandLineArguments default commandLineArguments nextToLast.
	defaultRemoteType := CommandLineArguments default commandLineArguments last asSymbol.

	Iceberg remoteTypeSelector: defaultRemoteType.

	Metacello new
		baseline: 'GToolkitVMMakerAddOns';
			repository: 'tonel://' , path , '/pharo-vm-addons';
			load.
]
