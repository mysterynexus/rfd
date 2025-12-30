package rfd;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.ContentResolver;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.ParcelFileDescriptor;
import android.provider.DocumentsContract;
import android.provider.OpenableColumns;

import androidx.activity.result.ActivityResult;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public class RfdHelper {
    private static RfdHelper instance;
    private AppCompatActivity activity;
    private ActivityResultLauncher<Intent> openDocumentLauncher;
    private ActivityResultLauncher<Intent> openDocumentTreeLauncher;
    private ActivityResultLauncher<Intent> createDocumentLauncher;

    private final ConcurrentHashMap<Long, RequestContext> pendingRequests = new ConcurrentHashMap<>();

    private static class RequestContext {
        final long requestId;
        final RequestType type;
        RequestContext(long id, RequestType t) { requestId = id; type = t; }
    }

    private enum RequestType { OPEN_FILE, OPEN_FOLDER, CREATE_FILE }

    private long currentOpenFileRequest;
    private long currentOpenFolderRequest;
    private long currentCreateFileRequest;

    public static void init(AppCompatActivity activity) {
        instance = new RfdHelper();
        instance.activity = activity;
        instance.registerLaunchers();
        nativeInit();
    }

    private static native void nativeInit();

    public static RfdHelper getInstance() {
        return instance;
    }

    private void registerLaunchers() {
        openDocumentLauncher = activity.registerForActivityResult(
            new ActivityResultContracts.StartActivityForResult(),
            this::onOpenDocumentResult
        );
        openDocumentTreeLauncher = activity.registerForActivityResult(
            new ActivityResultContracts.StartActivityForResult(),
            this::onOpenDocumentTreeResult
        );
        createDocumentLauncher = activity.registerForActivityResult(
            new ActivityResultContracts.StartActivityForResult(),
            this::onCreateDocumentResult
        );
    }

    public void pickFile(long requestId, String[] mimeTypes, boolean multiple) {
        currentOpenFileRequest = requestId;
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        if (mimeTypes.length == 1) {
            intent.setType(mimeTypes[0]);
        } else {
            intent.setType("*/*");
            intent.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes);
        }
        intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
        openDocumentLauncher.launch(intent);
    }

    public void pickFolder(long requestId) {
        currentOpenFolderRequest = requestId;
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        openDocumentTreeLauncher.launch(intent);
    }

    public void saveFile(long requestId, String mimeType, String fileName) {
        currentCreateFileRequest = requestId;
        Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType(mimeType);
        intent.putExtra(Intent.EXTRA_TITLE, fileName);
        createDocumentLauncher.launch(intent);
    }

    public void showMessageDialog(long requestId, String title, String message,
                                   String[] buttonLabels, int[] buttonResults) {
        activity.runOnUiThread(() -> {
            AlertDialog.Builder builder = new AlertDialog.Builder(activity);
            builder.setTitle(title);
            builder.setMessage(message);
            builder.setCancelable(true);
            builder.setOnCancelListener(dialog -> nativeOnMessageResult(requestId, 1));
            if (buttonLabels.length >= 1) {
                int result0 = buttonResults[0];
                builder.setPositiveButton(buttonLabels[0], (dialog, which) ->
                    nativeOnMessageResult(requestId, result0));
            }
            if (buttonLabels.length >= 2) {
                int result1 = buttonResults[1];
                builder.setNegativeButton(buttonLabels[1], (dialog, which) ->
                    nativeOnMessageResult(requestId, result1));
            }
            if (buttonLabels.length >= 3) {
                int result2 = buttonResults[2];
                builder.setNeutralButton(buttonLabels[2], (dialog, which) ->
                    nativeOnMessageResult(requestId, result2));
            }
            builder.show();
        });
    }

    private void onOpenDocumentResult(ActivityResult result) {
        long requestId = currentOpenFileRequest;
        if (result.getResultCode() == Activity.RESULT_OK && result.getData() != null) {
            ArrayList<String> paths = new ArrayList<>();
            Intent data = result.getData();
            if (data.getClipData() != null) {
                int count = data.getClipData().getItemCount();
                for (int i = 0; i < count; i++) {
                    Uri uri = data.getClipData().getItemAt(i).getUri();
                    String path = copyUriToCache(uri);
                    if (path != null) paths.add(path);
                }
            } else if (data.getData() != null) {
                String path = copyUriToCache(data.getData());
                if (path != null) paths.add(path);
            }
            if (paths.isEmpty()) {
                nativeOnCancelled(requestId);
            } else {
                nativeOnFilesSelected(requestId, paths.toArray(new String[0]));
            }
        } else {
            nativeOnCancelled(requestId);
        }
    }

    private void onOpenDocumentTreeResult(ActivityResult result) {
        long requestId = currentOpenFolderRequest;
        if (result.getResultCode() == Activity.RESULT_OK && result.getData() != null) {
            Uri treeUri = result.getData().getData();
            if (treeUri != null) {
                activity.getContentResolver().takePersistableUriPermission(
                    treeUri,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                );
                String docId = DocumentsContract.getTreeDocumentId(treeUri);
                Uri docUri = DocumentsContract.buildDocumentUriUsingTree(treeUri, docId);
                nativeOnFilesSelected(requestId, new String[]{docUri.toString()});
            } else {
                nativeOnCancelled(requestId);
            }
        } else {
            nativeOnCancelled(requestId);
        }
    }

    private void onCreateDocumentResult(ActivityResult result) {
        long requestId = currentCreateFileRequest;
        if (result.getResultCode() == Activity.RESULT_OK && result.getData() != null) {
            Uri uri = result.getData().getData();
            if (uri != null) {
                activity.getContentResolver().takePersistableUriPermission(
                    uri,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                );
                String path = createWritableFile(uri);
                if (path != null) {
                    nativeOnSaveFileSelected(requestId, path);
                } else {
                    nativeOnCancelled(requestId);
                }
            } else {
                nativeOnCancelled(requestId);
            }
        } else {
            nativeOnCancelled(requestId);
        }
    }

    private String copyUriToCache(Uri uri) {
        try {
            ContentResolver resolver = activity.getContentResolver();
            String originalName = getFileName(uri);
            String uniqueName = UUID.randomUUID().toString() + "_" +
                (originalName != null ? originalName : "file");
            File cacheDir = new File(activity.getCacheDir(), "rfd");
            if (!cacheDir.exists() && !cacheDir.mkdirs()) return null;
            File cacheFile = new File(cacheDir, uniqueName);
            try (InputStream in = resolver.openInputStream(uri);
                 OutputStream out = new FileOutputStream(cacheFile)) {
                if (in == null) return null;
                byte[] buffer = new byte[8192];
                int len;
                while ((len = in.read(buffer)) != -1) {
                    out.write(buffer, 0, len);
                }
            }
            return cacheFile.getAbsolutePath();
        } catch (Exception e) {
            return null;
        }
    }

    private String createWritableFile(Uri uri) {
        try {
            ContentResolver resolver = activity.getContentResolver();
            String originalName = getFileName(uri);
            String uniqueName = UUID.randomUUID().toString() + "_" +
                (originalName != null ? originalName : "file");
            File cacheDir = new File(activity.getCacheDir(), "rfd_save");
            if (!cacheDir.exists() && !cacheDir.mkdirs()) return null;
            File localFile = new File(cacheDir, uniqueName);
            if (!localFile.createNewFile()) return null;
            storeSaveMapping(localFile.getAbsolutePath(), uri.toString());
            return localFile.getAbsolutePath();
        } catch (Exception e) {
            return null;
        }
    }

    private void storeSaveMapping(String localPath, String contentUri) {
        try {
            File mappingDir = new File(activity.getCacheDir(), "rfd_mappings");
            if (!mappingDir.exists()) mappingDir.mkdirs();
            String hash = String.valueOf(localPath.hashCode());
            File mappingFile = new File(mappingDir, hash);
            try (FileOutputStream out = new FileOutputStream(mappingFile)) {
                out.write(contentUri.getBytes("UTF-8"));
            }
        } catch (Exception ignored) {}
    }

    public static String getSaveUriForPath(String localPath) {
        if (instance == null) return null;
        try {
            File mappingDir = new File(instance.activity.getCacheDir(), "rfd_mappings");
            String hash = String.valueOf(localPath.hashCode());
            File mappingFile = new File(mappingDir, hash);
            if (!mappingFile.exists()) return null;
            byte[] bytes = new byte[(int) mappingFile.length()];
            try (java.io.FileInputStream in = new java.io.FileInputStream(mappingFile)) {
                in.read(bytes);
            }
            return new String(bytes, "UTF-8");
        } catch (Exception e) {
            return null;
        }
    }

    public static boolean syncSaveFile(String localPath) {
        if (instance == null) return false;
        String uriString = getSaveUriForPath(localPath);
        if (uriString == null) return false;
        try {
            Uri uri = Uri.parse(uriString);
            ContentResolver resolver = instance.activity.getContentResolver();
            try (InputStream in = new java.io.FileInputStream(localPath);
                 OutputStream out = resolver.openOutputStream(uri, "wt")) {
                if (out == null) return false;
                byte[] buffer = new byte[8192];
                int len;
                while ((len = in.read(buffer)) != -1) {
                    out.write(buffer, 0, len);
                }
            }
            new File(localPath).delete();
            File mappingDir = new File(instance.activity.getCacheDir(), "rfd_mappings");
            new File(mappingDir, String.valueOf(localPath.hashCode())).delete();
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private String getFileName(Uri uri) {
        String result = null;
        if ("content".equals(uri.getScheme())) {
            try (Cursor cursor = activity.getContentResolver()
                    .query(uri, null, null, null, null)) {
                if (cursor != null && cursor.moveToFirst()) {
                    int index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                    if (index >= 0) result = cursor.getString(index);
                }
            } catch (Exception ignored) {}
        }
        if (result == null) {
            result = uri.getLastPathSegment();
        }
        return result;
    }

    private static native void nativeOnFilesSelected(long requestId, String[] paths);
    private static native void nativeOnCancelled(long requestId);
    private static native void nativeOnMessageResult(long requestId, int resultCode);
    private static native void nativeOnSaveFileSelected(long requestId, String path);
}
