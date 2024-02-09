package kyazuki;

import java.time.Instant;
import java.time.format.DateTimeFormatterBuilder;
import java.util.Map;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.datatype.jsr310.ser.InstantSerializer;

/**
 * Minecraftのプロファイル設定ファイルを管理するクラス
 */
public class Profiles {
    /**
     * 解像度設定
     */
    public static class Resolution {
        public int height;
        public int width;
    }

    /** プロファイル設定 */
    public static class Profile {
        /** プロファイル作成日時 */
        public Instant created;
        /** ゲームディレクトリ */
        @JsonInclude(Include.NON_NULL)
        public String gameDir;
        /** アイコン画像 */
        public String icon;
        /** JVM引数 */
        @JsonInclude(Include.NON_NULL)
        public String javaArgs;
        /** Javaパス */
        @JsonInclude(Include.NON_NULL)
        public String javaDir;
        /** 最終起動日時 */
        public Instant lastUsed;
        /** バージョンID */
        public String lastVersionId;
        /** プロファイル名 */
        public String name;
        /** 解像度設定 */
        @JsonInclude(Include.NON_NULL)
        public Resolution resolution;
        /** JREバージョンをチェックするか？ */
        @JsonInclude(Include.NON_NULL)
        public boolean skipJreVersionCheck;
        /** タイプ */
        public String type;
    }

    /** 各プロファイル設定 */
    public Map<String, Profile> profiles;
    /** その他の設定 */
    public Map<String, Object> settings;
    /** バージョン */
    public int version;

    /**
     * InstantクラスをMinecraftのプロファイル設定ファイルの形式に変換するためのカスタムシリアライザ
     */
    public static class CustomInstantSerializer extends InstantSerializer {
        public CustomInstantSerializer() {
            super(InstantSerializer.INSTANCE, false, false,
                    new DateTimeFormatterBuilder().appendInstant(3).toFormatter());
        }
    }
}