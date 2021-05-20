package com.evernym.vcx.reactnative.rnindy;

import android.content.Context;

import com.facebook.react.bridge.Arguments;

import org.json.JSONObject;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;


public class RNIndyStaticData {
    public static final int REQUEST_WRITE_EXTERNAL_STORAGE = 501;
    public static String LOG_FILE_PATH = "";
    public static String ENCRYPTED_LOG_FILE_PATH = "";
    public static int MAX_ALLOWED_FILE_BYTES = 10000000;
    public static LogFileObserver logFileObserver = null;


    public static void initLoggerFile(final Context context) {
        // create the log file if it does not exist
        try {
            if(! new File(RNIndyStaticData.LOG_FILE_PATH).exists()) {
                new FileWriter(RNIndyStaticData.LOG_FILE_PATH).close();
            }
        } catch(IOException ex) {
            ex.printStackTrace();
            return;
        }

        // Now monitor the logFile and empty it out when it's size is
        // larger than MAX_ALLOWED_FILE_BYTES
        RNIndyStaticData.logFileObserver = new LogFileObserver(RNIndyStaticData.LOG_FILE_PATH, RNIndyStaticData.MAX_ALLOWED_FILE_BYTES);
        RNIndyStaticData.logFileObserver.startWatching();

        pl.brightinventions.slf4android.FileLogHandlerConfiguration fileHandler = pl.brightinventions.slf4android.LoggerConfiguration.fileLogHandler(context);
        fileHandler.setFullFilePathPattern(RNIndyStaticData.LOG_FILE_PATH);
        fileHandler.setRotateFilesCountLimit(1);
        // Prevent slf4android from rotating the log file as we will handle that. The
        // way that we prevent slf4android from rotating the log file is to set the log
        // file size limit to 1 million bytes higher that our MAX_ALLOWED_FILE_BYTES
        fileHandler.setLogFileSizeLimitInBytes(RNIndyStaticData.MAX_ALLOWED_FILE_BYTES + 1000000);
        pl.brightinventions.slf4android.LoggerConfiguration.configuration().addHandlerToRootLogger(fileHandler);

        // !!TODO: Remove the pl.brightinventions.slf4android.LoggerConfiguration.configuration() console logger

    }
}
