package com.evernym.sdk.vcx.vcx;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 13/06/18.
 */

public class MessageIsOutOfThreadException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Message is out of thread";


    public MessageIsOutOfThreadException()
    {
        super(message, ErrorCode.MESSAGE_IS_OUT_OF_THREAD.value());
    }
}