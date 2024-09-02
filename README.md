# Dedma
[Lire en français](README_fr.md)
### Release notes generator
A Command Line Interface (CLI) that generates release notes from your latest commits.  

## Display help
    dedma --help

## Specify the language for generating the release notes
### French
To generate the release notes in french add `LANG_FR` to the environment variables
For systems based on Unix
	LANG_FR=1 

For Windows PowerShell
	$Env:LANG_FR=1

### English
It generate the release notes in english by default. You don't have to do anything

## Get the commits from a file
    dedma input_file output file

## Get them directly from git
    dedma output_file
    
or 

    dedma

to generate the notes in the file `whats_new.md`


## Sample
Convert this  

    feat (Tower): added one tower type 
    fix: Made the projectile rotation and starting point related to tower and tank position as well as rotation 
    feat (Reward): Added two more rewards 
    update: Added more balance to the game 
    feat (Economy): Added money logic

Into this

    # New features
    ## Tower
    - added one tower type
    ## Reward
    - Added two more rewards
    ## Economy
    - Added money logic
    # Bug fix
    - Made the projectile rotation and starting point related to tower and tank position as well as rotation
    # Updates
    - Added more balance to the game

## Ideal commit structure
    kind (title): content
For `title` and `content` you can put whatever you want.  
Here are the supported `kind` right now in the order of appearance in the generated notes


kind  | full name | meaning
:---: | :---:  | ---
feat | New features  | a new feature is introduced with the changes  
fix | Bug fix  | a bug fix has occurred  
chore | Chore  | changes that do not relate to a fix or feature and don't modify src or test files (for example updating dependencies)  
refactor | Refactoring  | refactored code that neither fixes a bug nor adds a feature  
docs | Documentation  | updates to documentation such as a the README or other markdown files  
style | Code Style  | changes that do not affect the meaning of the code, likely related to code formatting such as white-space, missing semi-colons, and so on.  
test | Test  | including new or correcting previous tests
perf | Performances  | performance improvements  
ci | Continuous Integration (CI)  | continuous integration related  
build | Build System  | changes that affect the build system or external dependencies  
revert | Reverts  | reverts a previous commit  
update | Updates  | Any update  