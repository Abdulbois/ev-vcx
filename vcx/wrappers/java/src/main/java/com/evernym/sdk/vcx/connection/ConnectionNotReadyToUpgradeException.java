package com.evernym.sdk.vcx.connection;

import com.evernym.sdk.vcx.ErrorCode;
import com.evernym.sdk.vcx.VcxException;

public class ConnectionNotReadyToUpgradeException extends VcxException
{
    private static final long serialVersionUID = 3294831240096535507L;
    private final static String message = "Connection not ready to upgrade. Enterprise side has not upgraded connection yet.";


    public ConnectionNotReadyToUpgradeException()
    {
        super(message, ErrorCode.CONNECTION_NOT_READY_TO_UPGRADE.value());
    }
}
