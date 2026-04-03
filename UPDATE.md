## New CLI

- Option to remove a package from the repository
  - This can be done with `repo-remove` but having the option here, will make sure it automatically signs / updates the repository db
      signature, according to settings
- Option to update the repository db signature
  - In the case that the repository signature fails to be applied for any reason, this option will manually update it.
- Option to re-build all the packages in the repository
  - In case the signing settings of the repository have changed, and the packages need to be signed or unsigned.
- Remove orphaned
  - Remove any packages from the repository that are no longer part of the AUR. Packages can be moved from AUR to normal repositories.
  - This will clean up those packages.
- Option to create a new repository.
  - Creates an initial repository (Right now this is a manual step via repo-add) 
- A better way to handle unknown keys when building
- Ability to run a command directly in the docker image
  - Either to test something out, or perform something that isn't supported by the CLI. 