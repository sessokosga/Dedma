Dedma v0.1.2
Release notes generator
A Command Line Interface (CLI) that generates release notes from your latest commits. 

The release notes are generated in english by default.

To generate the release notes in french add `LANG_FR` to the environment variables
For systems based on Unix
	LANG_FR=1 

For Windows PowerShell
	$Env:LANG_FR=1

Get the commits from a file
    dedma input_file output file

Get them directly from git
    dedma output_file
    
        or 

    dedma

to generate the notes in the file `whats_new.md`

Ideal commit structure
    kind (title): content
For `title` and `content` you can put whatever you want.  
Here are the supported `kind` right now in the order of appearance in the generated notes

 ______________________________________

|   kind   | full name                   |
| :------: | --------------------------- |
|   feat   | New features                |
|   fix    | Bug fix                     |
|  chore   | Chore                       |
| refactor | Refactoring                 |
|   docs   | Documentation               |
|  style   | Code Style                  |
|   test   | Test                        |
|   perf   | Performances                |
|    ci    | Continuous Integration (CI) |
|  build   | Build System                |
|  revert  | Reverts                     |
|  update  | Updates                     |
 ______________________________________
