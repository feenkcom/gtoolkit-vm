Author useAuthor: 'installVMMaker' during: [ | path defaultRemoteType |
	path := CommandLineArguments new commandLineArguments nextToLast.
	defaultRemoteType := CommandLineArguments new commandLineArguments last asSymbol.

	Iceberg remoteTypeSelector: defaultRemoteType.

	Metacello new
		baseline: 'GToolkitVMMakerAddOns';
			repository: 'tonel://' , path , '/pharo-vm-addons';
			load.
]
