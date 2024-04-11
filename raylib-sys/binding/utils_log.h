#if defined(__cplusplus)
extern "C"
{ // Prevents name mangling of functions
#endif

    void setLogCallbackWrapper(void); // enable the call-back
#ifdef _WIN32
    void custom_trace_log_callback(char * logType, const char *text, int len);
#else
    void custom_trace_log_callback(int logType, const char *text, int len);
#endif
#if defined(__cplusplus)
}
#endif
