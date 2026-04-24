/*
* Preprocessor conditional example
*/

#ifdef DEBUG_MODE
   MEMVAR cDebugPath
   PROCEDURE DebugLog()
      ? 'Debug enabled'
   RETURN
#endif

PUBLIC nVersion := 100

FUNCTION GetVersion()
RETURN nVersion
