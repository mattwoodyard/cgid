# CGID

A modern take on publishing adhoc tools via a service interface


## What does it do

It runs a webserver, turns an HTTP request into a json document and then executes a script passing that json document on stdin. It takes the command's stdout, parses it as a json document and returns it as the response to the original client. 


## TODO
  - [] tls, acme support
  - [] request based authentication via defined script endpoint
  - [] h2 support 
  - [] failure webhooks
  - [] object storage support for script_root

## PRO
  - There's a premium version available which has support for mtls, gssapi, multi user execution and script versioning
