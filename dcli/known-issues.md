* Manifest not completed
* Secret list needs default (works only if --app or --application is provided)
* Secret --app and --application show also the other
* Secret: "dcli secret list" gives error, "dcli secrets" works
* Env find, when no results, strange empty table is printed
* Application diff not complete

## The following commands need attention

dcli -vvv application diff cmd
dcli -vvv bucket show cpr
dcli -vvv bucket show cpr --all
dcli -vvv bucket show cpr --status
dcli -vvv env find info
dcli -vvv env find info --app
dcli -vvv env find info --application
dcli -vvv manifest list
dcli -vvv manifest list
dcli -vvv manifest list --all
dcli -vvv manifest list --configuration
dcli -vvv manifest list --ids
dcli -vvv secret list --usage
dcli -vvv secret list --app
dcli -vvv secret list --application
dcli -vvv volume list --usage --app
dcli -vvv volume list --usage --application

