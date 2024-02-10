package kyazuki.dataclass;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.net.HttpURLConnection;
import java.net.URL;
import java.net.URLDecoder;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * 設定ファイルの内容を管理するクラス
 */
public class Config {
    /**
     * プロファイルの設定を管理するクラス
     */
    public static class Profile {
        public String name;
        public String icon;
        public String versionId;
        public String javaArgs;
    }

    /**
     * ダウンロードファイルの設定を管理する抽象クラス
     */
    public static abstract class DownloadFile {
        /** 名前 */
        public String name;
        /** ダウンロードURL */
        protected String url;
        /** ファイル名 (キャッシュ用) */
        @JsonIgnore
        protected String filename = null;

        protected abstract Path getDirectory();

        /**
         * ファイル名を取得する
         */
        public String getFileName() throws IOException {
            if (filename == null) {
                String[] urls = getRedirectedURL(url).split("/");
                filename = URLDecoder.decode(urls[urls.length - 1], "UTF-8");
            }
            return filename;
        }

        /**
         * ダウンロードを行う
         *
         * @return ダウンロードしたらtrue、すでにダウンロード済みならfalse
         * @throws IOException
         */
        public boolean download() throws IOException {
            Path directory = getDirectory();
            Path path = directory.resolve(getFileName());
            if (Files.exists(path)) {
                return false;
            }
            Files.createDirectories(directory);
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            HttpURLConnection con = null;
            try {
                con = (HttpURLConnection) new URL(url).openConnection();
                con.connect();
                if (con.getResponseCode() != HttpURLConnection.HTTP_OK) {
                    throw new IOException("Error response. code: " + con.getResponseCode());
                }
                con.getInputStream().transferTo(baos);
            } finally {
                if (con != null) {
                    con.disconnect();
                }
            }
            Files.copy(new ByteArrayInputStream(baos.toByteArray()), path, StandardCopyOption.REPLACE_EXISTING);
            return true;
        }

        /**
         * リダイレクト先のURLを取得する
         *
         * @param url
         * @return リダイレクト先URL
         * @throws IOException
         */
        private static String getRedirectedURL(String url) throws IOException {
            HttpURLConnection con = null;
            try {
                con = (HttpURLConnection) new URL(url).openConnection();
                con.setInstanceFollowRedirects(false);
                con.connect();
                if (con.getResponseCode() >= HttpURLConnection.HTTP_MULT_CHOICE && con
                        .getResponseCode() < HttpURLConnection.HTTP_BAD_REQUEST) {
                    String redirectUrl = con.getHeaderField("Location");
                    return getRedirectedURL(redirectUrl);
                }
                if (con.getResponseCode() != HttpURLConnection.HTTP_OK) {
                    throw new IOException("Error response. code: " + con.getResponseCode());
                }
                return url;
            } finally {
                if (con != null) {
                    con.disconnect();
                }
            }
        }
    }

    /**
     * ModLoaderのダウンロード設定を管理するクラス
     */
    public static class ModLoader extends DownloadFile {
        /** インストール完了後に自動で実行するか */
        public boolean autoOpen;

        @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
        public ModLoader(@JsonProperty("name") String name, @JsonProperty("url") String url,
                @JsonProperty("autoOpen") boolean autoOpen) {
            this.name = name;
            this.url = url;
            this.autoOpen = autoOpen;
        }

        @Override
        protected Path getDirectory() {
            return Path.of(".");
        }
    }

    /**
     * CurseForge上のModのダウンロード設定を管理するクラス
     */
    public static class CurseForgeMod extends DownloadFile {
        @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
        public CurseForgeMod(@JsonProperty("name") String name, @JsonProperty("modId") int modId,
                @JsonProperty("fileId") int fileId) {
            this.name = name;
            this.url = String.format("https://www.curseforge.com/api/v1/mods/%d/files/%d/download", modId, fileId);
        }

        @Override
        protected Path getDirectory() {
            return Path.of("mods");
        }
    }

    /** プロファイル設定 */
    @JsonProperty("Profile")
    public Profile profile;
    /** ModLoaderのダウンロード設定 */
    @JsonProperty("ModLoader")
    public ModLoader modLoader;
    /** 各Mod(CurseForge)のダウンロード設定 */
    @JsonProperty("CurseForgeMods")
    public CurseForgeMod[] curseForgeMods;
}