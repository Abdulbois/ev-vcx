package com.evernym.sdk.vcx.connection;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 05/06/18.
 */

public class ConnectionDoesNotExistException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Connection does not exist.";


    public ConnectionDoesNotExistException()
    {
        super(message, ErrorCode.CONNECTION_DOES_NOT_EXIST.value());
    }
}
