Stdio stdout << 'Patch LGitLibrary...'.

LGitLibrary compile:
'unix64LibraryName
	<gtPharoPatch: #Pharo>

	^ FFIUnix64LibraryFinder findAnyLibrary: #(
	    ''libgit2.so''
		"This name is wrong, but some versions of the VM has this library shipped with the bad name"
		''libgit2.1.0.0.so''
		''libgit2.so.1.0.0''
		''libgit2.so.1.0''
		''libgit2.so.1.1''
		''libgit2.so.0.25.1'')'
	classified: 'gt-vmmaker-patch'.

LGitLibrary compile:
'win32LibraryName
	<gtPharoPatch: #Pharo>

    ^ FFIWindowsLibraryFinder findAnyLibrary: #(''git2.dll'' ''libgit2-1-0-0.dll'' ''libgit2.dll'')'
	classified: 'gt-vmmaker-patch'.

Stdio stdout << 'done'; lf.

Stdio stdout << 'Patch FFILibraryFinders...'.

FFIUnixLibraryFinder compile:
'basePaths
	<gtPharoPatch: #Pharo>

	^ {
	(Smalltalk vm directory asFileReference parent / ''lib'') pathString.
	Smalltalk imageDirectory fullName.
	Smalltalk vm directory }'
  classified: 'gt-vmmaker-patch'.

FFIWindowsLibraryFinder compile:
'basePaths

	^ {
	(Smalltalk vm directory asFileReference parent / ''lib'') pathString.
	Smalltalk imageDirectory fullName.
	Smalltalk vm directory }'
  classified: 'gt-vmmaker-patch'.

Stdio stdout << 'done'; lf.
