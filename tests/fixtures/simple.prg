/*
* Simple test file
*/

MEMVAR usuario, nSerial

PROCEDURE TestSimple()
   LOCAL cNome := ''
   cNome := 'John'
RETURN

FUNCTION GetUser()
   LOCAL oUser := NIL
   oUser := FetchUser()
RETURN oUser
