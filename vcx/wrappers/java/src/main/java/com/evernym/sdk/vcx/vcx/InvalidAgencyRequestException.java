package com.evernym.sdk.vcx.vcx;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 05/06/18.
 */

public class InvalidAgencyRequestException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "The message submitted on the Agency has an invalid format or field value";


    public InvalidAgencyRequestException()
    {
        super(message, ErrorCode.INVALID_AGENCY_REQUEST.value());
    }
}