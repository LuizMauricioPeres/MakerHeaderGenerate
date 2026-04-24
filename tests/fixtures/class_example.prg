/*
* Class definition example
*/

CLASS MyClass
   VAR nId EXPORTED
   VAR cName HIDDEN
   VAR lActive PROTECTED

   METHOD New()
   METHOD Init( nId, cName )
   ACCESS getId()
   ASSIGN setId( nValue )

ENDCLASS

METHOD MyClass:New()
   LOCAL oInstance := ::New()
RETURN oInstance

METHOD MyClass:Init( nId, cName )
   ::nId := nId
   ::cName := cName
RETURN

METHOD MyClass:getId()
RETURN ::nId

METHOD MyClass:setId( nValue )
   ::nId := nValue
RETURN
