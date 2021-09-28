package com.evernym.sdk.vcx.connection;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

public class ConnectionNotCompletedException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Connection used for sending a message is not in the completed state.";


    public ConnectionNotCompletedException()
    {
        super(message, ErrorCode.CONNECTION_NOT_COMPLETED.value());
    }
}
