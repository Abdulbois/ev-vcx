package com.evernym.sdk.vcx.vcx;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

/**
 * Created by abdussami on 13/06/18.
 */

public class InvalidCredentialOfferException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Invalid Credential Offerz";


    public InvalidCredentialOfferException()
    {
        super(message, ErrorCode.INVALID_CREDENTIAL_OFFER.value());
    }
}
