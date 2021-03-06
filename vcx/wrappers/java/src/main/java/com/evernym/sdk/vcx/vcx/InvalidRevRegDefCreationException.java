package com.evernym.sdk.vcx.vcx;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 05/06/18.
 */

public class InvalidRevRegDefCreationException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Failed to create Revocation Registration Definition";


    public InvalidRevRegDefCreationException()
    {
        super(message, ErrorCode.INVALID_REV_REG_DEF_CREATION.value());
    }
}