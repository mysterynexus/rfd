package io.github.polymeilex.rfd;

import android.app.Activity;
import android.content.ClipData;
import android.content.ContentResolver;
import android.content.Context;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.OpenableColumns;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

public final class RfdBridge {
    private RfdBridge() {}

    public static void openDocument(Context context, String[] mimeTypes, boolean multiple, String title, int requestCode) {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        if (mimeTypes != null && mimeTypes.length > 0) {
            intent.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes);
        }
        intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
        if (title != null) {
            intent.putExtra(Intent.EXTRA_TITLE, title);
        }
        ProxyActivity.startForResult(context, intent, requestCode);
    }

    public static native void onActivityResultCallback(int requestCode, int resultCode, String[] uris);

    public static String[] copyUrisToCache(Context context, String[] uriStrings) {
        if (uriStrings == null) return null;

        List<String> out = new ArrayList<>(uriStrings.length);
        ContentResolver resolver = context.getContentResolver();
        File cacheDir = context.getCacheDir();
        for (String s : uriStrings) {
            if (s == null) continue;
            Uri uri = Uri.parse(s);
            String displayName = queryDisplayName(resolver, uri);
            if (displayName == null || displayName.trim().isEmpty()) {
                displayName = "rfd_" + UUID.randomUUID();
            }
            File dest = uniqueFile(cacheDir, displayName);
            try (InputStream in = resolver.openInputStream(uri); FileOutputStream outStream = new FileOutputStream(dest)) {
                if (in == null) continue;
                byte[] buf = new byte[64 * 1024];
                int read;
                while ((read = in.read(buf)) != -1) {
                    outStream.write(buf, 0, read);
                }
                out.add(dest.getAbsolutePath());
            } catch (IOException e) {
                // Skip on error
            }
        }
        return out.toArray(new String[0]);
    }

    private static String queryDisplayName(ContentResolver resolver, Uri uri) {
        Cursor cursor = null;
        try {
            cursor = resolver.query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null);
            if (cursor != null && cursor.moveToFirst()) {
                int idx = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                if (idx >= 0) return cursor.getString(idx);
            }
        } catch (Throwable t) {
            // ignore
        } finally {
            if (cursor != null) cursor.close();
        }
        return null;
    }

    private static File uniqueFile(File dir, String baseName) {
        File f = new File(dir, baseName);
        if (!f.exists()) return f;
        String name = baseName;
        String ext = "";
        int dot = baseName.lastIndexOf('.');
        if (dot > 0 && dot < baseName.length() - 1) {
            name = baseName.substring(0, dot);
            ext = baseName.substring(dot);
        }
        int i = 1;
        while (true) {
            File attempt = new File(dir, name + " (" + i + ")" + ext);
            if (!attempt.exists()) return attempt;
            i++;
        }
    }

    public static class ProxyActivity extends Activity {
        private static Intent pendingIntent;
        private static int pendingRequestCode;

        public static void startForResult(Context context, Intent intent, int requestCode) {
            pendingIntent = intent;
            pendingRequestCode = requestCode;
            Intent proxy = new Intent(context, ProxyActivity.class);
            proxy.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            context.startActivity(proxy);
        }

        @Override
        protected void onCreate(Bundle savedInstanceState) {
            super.onCreate(savedInstanceState);
            if (pendingIntent == null) {
                finish();
                return;
            }
            try {
                startActivityForResult(pendingIntent, 1001);
            } catch (Throwable t) {
                finish();
            }
        }

        @Override
        protected void onActivityResult(int requestCode, int resultCode, Intent data) {
            super.onActivityResult(requestCode, resultCode, data);
            if (requestCode == 1001) {
                String[] uris = null;
                if (resultCode == RESULT_OK && data != null) {
                    List<String> all = new ArrayList<>();
                    Uri single = data.getData();
                    if (single != null) {
                        all.add(single.toString());
                    } else {
                        ClipData clip = data.getClipData();
                        if (clip != null) {
                            for (int i = 0; i < clip.getItemCount(); i++) {
                                Uri u = clip.getItemAt(i).getUri();
                                if (u != null) all.add(u.toString());
                            }
                        }
                    }
                    uris = all.toArray(new String[0]);
                }
                try {
                    RfdBridge.onActivityResultCallback(pendingRequestCode, resultCode, uris);
                } catch (Throwable ignored) {
                }
                pendingIntent = null;
                finish();
            }
        }
    }
}

