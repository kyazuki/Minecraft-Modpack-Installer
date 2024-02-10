module kyazuki {
    requires transitive java.logging;

    requires com.fasterxml.jackson.databind;
    requires com.fasterxml.jackson.dataformat.yaml;
    requires com.fasterxml.jackson.datatype.jsr310;

    requires javafx.controls;
    requires transitive javafx.graphics;
    requires javafx.fxml;

    opens kyazuki to javafx.fxml;

    exports kyazuki;
}