package com.evernym.sdk.vcx.credential;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 13/06/18.
 */

public class InvalidCredentialDefJsonException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "VCX Exception";


    public InvalidCredentialDefJsonException()
    {
        super(message, ErrorCode.CREDENTIAL_DEFINITION_NOT_FOUND.value());
    }
}