package kyazuki.dataclass;

import java.util.Map;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;

/**
 * Minecraftのプロファイル設定ファイルを管理するクラス
 */
public class Profiles {
    /**
     * 解像度設定
     */
    public static class Resolution {
        /** 高さ */
        public int height;
        /** 幅 */
        public int width;
    }

    /** プロファイル設定 */
    public static class Profile {
        /** プロファイル作成日時 */
        public String created;
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
        public String lastUsed;
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
}