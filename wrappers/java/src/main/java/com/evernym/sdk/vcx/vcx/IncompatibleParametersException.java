package com.evernym.sdk.vcx.vcx;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

public class IncompatibleParametersException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Passed a combination of incompatible parameters";


    public IncompatibleParametersException()
    {
        super(message, ErrorCode.INCOMPATIBLE_PARAMETERS.value());
    }
}