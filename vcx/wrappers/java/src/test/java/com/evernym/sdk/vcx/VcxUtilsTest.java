package com.evernym.sdk.vcx;

import com.evernym.sdk.vcx.utils.UtilsApi;
import com.evernym.sdk.vcx.vcx.VcxApi;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.concurrent.ExecutionException;

import static org.junit.jupiter.api.Assertions.assertNotNull;

public class VcxUtilsTest {
    @BeforeEach
    void setup() throws Exception {
        System.setProperty(org.slf4j.impl.SimpleLogger.DEFAULT_LOG_LEVEL_KEY, "DEBUG");
        if (!TestHelper.vcxInitialized) {
            TestHelper.getResultFromFuture(VcxApi.vcxInit(TestHelper.VCX_CONFIG_TEST_MODE));
            TestHelper.vcxInitialized = true;
        }
    }

    @Test
    @DisplayName("endorse transaction")
    void vcxEndorseTransaction() throws VcxException, ExecutionException, InterruptedException {
        String transactionJson = "{\"req_id\":1, \"identifier\": \"EbP4aYNeTHL6q385GuVpRV\", \"signature\": \"gkVDhwe2\", \"endorser\": \"NcYxiDXkpYi6ov5FcYDi1e\"}";
        TestHelper.getResultFromFuture(UtilsApi.vcxEndorseTransaction(transactionJson));
    }

    @Test
    @DisplayName("get message")
    void vcxGetMessage() throws VcxException, ExecutionException, InterruptedException {
        String message = TestHelper.getResultFromFuture(UtilsApi.vcxGetMessage("abc"));
        assertNotNull(message);

    }

    @Test
    @DisplayName("fetch public entities")
    void vcxFetchPublicEntities() throws VcxException, ExecutionException, InterruptedException {
        TestHelper.getResultFromFuture(UtilsApi.vcxFetchPublicEntities());
    }
}
